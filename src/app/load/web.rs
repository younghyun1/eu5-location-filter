//! Browser loading: the embedded bundles decode synchronously once the first frame is painted,
//! and language and font packs are fetched relative to the frame document through the host.

use std::sync::Arc;

use iced::Task;
use js_sys::{Function, Promise};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

use super::packs::{font_message, pack_fetch_path, register_font};
use super::{LoadedData, prepare};
use crate::app::handoff::Handoff;
use crate::app::host;
use crate::app::i18n::{FontPack, Text};
use crate::app::message::{LoadMessage, Message};
use crate::app::state::LaunchPaths;
use crate::lang_pack::GameLanguage;
use crate::model::Dataset;
use crate::{AppError, embedded};

/// Decodes the embedded bundles now (called after `LoadMessage::Painted`) and answers with
/// `LoadMessage::Finished` for `generation`; `launch` and `force` have no meaning here.
///
/// The decode blocks the page's only thread, so a progress message is queued first and the
/// decode waits for the next paint; otherwise the loading card would never be shown.
pub(crate) fn start(_launch: &LaunchPaths, generation: u64, _force: bool) -> Task<Message> {
    let progress = Task::done(Message::Load(LoadMessage::Progress {
        generation,
        stage: Text::LoadEmbedded,
        fraction: 0.0,
    }));
    let decode = Task::future(async move {
        after_next_paint().await;
        Message::Load(LoadMessage::Finished {
            generation,
            result: decode_embedded(),
        })
    });
    progress.chain(decode)
}

/// Fetches `lang/<pack_file_name(language, build_id)>` with `PACK_MAX_BYTES` and answers with
/// `LoadMessage::LanguagePackFetched`; update validates the bytes.
pub(crate) fn language_pack(language: GameLanguage, dataset: &Arc<Dataset>) -> Task<Message> {
    let Some(path) = pack_fetch_path(language, dataset.stored.build_id) else {
        return Task::done(pack_message(
            language,
            Err(AppError::InvalidData(format!(
                "no language pack exists for {}",
                language.code()
            ))),
        ));
    };
    Task::perform(
        host::fetch_asset_bytes(path, host::PACK_MAX_BYTES),
        move |result| pack_message(language, result),
    )
}

/// Fetches `FontPack::fetch_path`, registers it with `iced::font::load`, and answers with
/// `LoadMessage::FontPackLoaded`.
pub(crate) fn font_pack(pack: FontPack) -> Task<Message> {
    let fetch = host::fetch_asset_bytes(pack.fetch_path().to_owned(), host::PACK_MAX_BYTES);
    Task::future(fetch).then(move |result| match result {
        Ok(bytes) if !bytes.is_empty() => register_font(pack, bytes),
        Ok(_empty) => Task::done(font_message(
            pack,
            Err(AppError::InvalidData(format!(
                "font pack {} is empty",
                pack.file_stem()
            ))),
        )),
        Err(error) => Task::done(font_message(pack, Err(error))),
    })
}

/// Decodes the embedded bundles and builds the engine.
fn decode_embedded() -> Result<Handoff<LoadedData>, Arc<AppError>> {
    embedded::load()
        .and_then(|(dataset, index)| prepare(dataset, Some(index)))
        .map(Handoff::new)
        .map_err(Arc::new)
}

/// The `LanguagePackFetched` message for a fetch outcome.
fn pack_message(language: GameLanguage, result: Result<Vec<u8>, AppError>) -> Message {
    Message::Load(LoadMessage::LanguagePackFetched {
        language,
        result: result.map(Arc::from).map_err(Arc::new),
    })
}

/// Resolves after the browser has painted the frame that follows the call: an animation frame
/// callback runs before that paint, and the zero-delay timer it schedules runs after it.
/// Any failure to schedule resolves at once, which only costs the loading card a frame.
async fn after_next_paint() {
    let promise = Promise::new(&mut |resolve: Function, _reject: Function| {
        wait_for_frame(&resolve);
    });
    // The promise only resolves, so the outcome carries nothing to handle.
    let _ = JsFuture::from(promise).await;
}

/// Schedules `defer_to_timer` for the next animation frame.
fn wait_for_frame(resolve: &Function) {
    let Some(window) = web_sys::window() else {
        settle(resolve);
        return;
    };
    let pending = resolve.clone();
    let callback = Closure::once_into_js(move || defer_to_timer(&pending));
    if window
        .request_animation_frame(callback.unchecked_ref())
        .is_err()
    {
        settle(resolve);
    }
}

/// Resolves from a zero-delay timer, which the browser runs after the frame is painted.
fn defer_to_timer(resolve: &Function) {
    let Some(window) = web_sys::window() else {
        settle(resolve);
        return;
    };
    let pending = resolve.clone();
    let callback = Closure::once_into_js(move || settle(&pending));
    if window
        .set_timeout_with_callback_and_timeout_and_arguments_0(callback.unchecked_ref(), 0)
        .is_err()
    {
        settle(resolve);
    }
}

/// Calls a promise resolver; a throwing resolver has nothing further to report.
fn settle(resolve: &Function) {
    let _ = resolve.call0(&JsValue::UNDEFINED);
}
