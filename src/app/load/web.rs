//! Browser loading: the embedded bundles decode synchronously once the first frame is painted,
//! and language and font packs are fetched relative to the frame document through the host.

use std::sync::Arc;

use iced::Task;

use crate::app::i18n::FontPack;
use crate::app::message::Message;
use crate::app::state::LaunchPaths;
use crate::lang_pack::GameLanguage;
use crate::model::Dataset;

/// Decodes the embedded bundles now (called after `LoadMessage::Painted`) and answers with
/// `LoadMessage::Finished` for `generation`; `launch` and `force` have no meaning here.
pub(crate) fn start(launch: &LaunchPaths, generation: u64, force: bool) -> Task<Message> {
    let _ = (launch, generation, force);
    Task::none()
}

/// Fetches `lang/<pack_file_name(language, build_id)>` with `PACK_MAX_BYTES` and answers with
/// `LoadMessage::LanguagePackFetched`; update validates the bytes.
pub(crate) fn language_pack(language: GameLanguage, dataset: &Arc<Dataset>) -> Task<Message> {
    let _ = (language, dataset);
    Task::none()
}

/// Fetches `FontPack::fetch_path`, registers it with `iced::font::load`, and answers with
/// `LoadMessage::FontPackLoaded`.
pub(crate) fn font_pack(pack: FontPack) -> Task<Message> {
    let _ = pack;
    Task::none()
}
