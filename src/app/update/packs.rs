//! Language and font pack requests and results. Every request is bounded by the fixed-size
//! bookkeeping in `PackState`, and a permanent failure is never fetched again in a session.

use std::sync::Arc;

use iced::Task;

use super::filters::after_language_change;
use crate::AppError;
use crate::app::handoff::Handoff;
use crate::app::i18n::FontPack;
use crate::app::load;
use crate::app::message::Message;
use crate::app::state::{App, Notice, PackFailure};
use crate::lang_pack::{GameLanguage, ValidatedLanguagePack};

/// Requests the font pack and game language pack the current locale needs.
pub(super) fn request_packs(app: &mut App) -> Task<Message> {
    let font = request_font(app);
    let language = request_language(app);
    Task::batch([font, language])
}

/// Requests the locale's script font once; fonts cannot be unloaded, so a loaded pack stays.
fn request_font(app: &mut App) -> Task<Message> {
    let Some(pack) = app.locale.font_pack() else {
        return Task::none();
    };
    let index = pack.index();
    let loaded = app.packs.fonts_loaded.get(index).copied().unwrap_or(true);
    let pending = app.packs.fonts_pending.get(index).copied().unwrap_or(true);
    if loaded || pending {
        return Task::none();
    }
    if let Some(slot) = app.packs.fonts_pending.get_mut(index) {
        *slot = true;
    }
    load::font_pack(pack)
}

/// Switches game text to the locale's language: English drops any pack at once, another
/// language is requested unless it is already active, loading, or failed permanently.
fn request_language(app: &mut App) -> Task<Message> {
    let wanted = app.locale.game_language();
    let (active, dataset) = match app.data.as_ref() {
        Some(data) => (data.engine.active_language(), Arc::clone(&data.dataset)),
        None => return Task::none(),
    };
    if wanted == active {
        return Task::none();
    }
    if wanted.is_embedded() {
        revert_to_english(app);
        return Task::none();
    }
    if app.packs.pending_language == Some(wanted) {
        return Task::none();
    }
    match app.packs.language_failure(wanted) {
        Some(PackFailure::Permanent | PackFailure::Mismatch) => {
            revert_to_english(app);
            return Task::none();
        }
        Some(PackFailure::Retryable) | None => {}
    }
    app.packs.pending_language = Some(wanted);
    load::language_pack(wanted, &dataset)
}

/// Browser: validates fetched pack bytes inside update and installs the pack.
pub(super) fn fetched(
    app: &mut App,
    language: GameLanguage,
    result: Result<Arc<[u8]>, Arc<AppError>>,
) -> Task<Message> {
    if !accepts_result(app, language) {
        return Task::none();
    }
    match result {
        Ok(bytes) => {
            let validated = match app.data.as_ref() {
                Some(data) => load::validate_pack_bytes(bytes.as_ref(), &data.dataset, language),
                None => return Task::none(),
            };
            match validated {
                Ok(pack) => install(app, language, pack),
                Err(error) => failed(app, language, &error),
            }
        }
        Err(error) => failed(app, language, error.as_ref()),
    }
    Task::none()
}

/// Desktop: installs a pack validated off the UI thread.
pub(super) fn validated(
    app: &mut App,
    language: GameLanguage,
    result: Result<Handoff<ValidatedLanguagePack>, Arc<AppError>>,
) -> Task<Message> {
    if !accepts_result(app, language) {
        return Task::none();
    }
    match result {
        Ok(handoff) => {
            // `None` means an earlier delivery of the same message already took the pack.
            if let Some(pack) = handoff.take() {
                install(app, language, pack);
            }
        }
        Err(error) => failed(app, language, error.as_ref()),
    }
    Task::none()
}

/// Records a font pack result; a failure shows the font notice and lets a later locale
/// change try again.
pub(super) fn font_loaded(
    app: &mut App,
    pack: FontPack,
    result: Result<(), Arc<AppError>>,
) -> Task<Message> {
    let index = pack.index();
    if let Some(slot) = app.packs.fonts_pending.get_mut(index) {
        *slot = false;
    }
    match result {
        Ok(()) => {
            if let Some(slot) = app.packs.fonts_loaded.get_mut(index) {
                *slot = true;
            }
        }
        Err(_failure) => app.overlay.notice = Some(Notice::FontPack),
    }
    Task::none()
}

/// Clears the pending mark and decides whether a result is still wanted: the locale may have
/// moved to another language, or the pack may already be active.
fn accepts_result(app: &mut App, language: GameLanguage) -> bool {
    if app.packs.pending_language == Some(language) {
        app.packs.pending_language = None;
    }
    let active = app.data.as_ref().map(|data| data.engine.active_language());
    language == app.locale.game_language() && active.is_some() && active != Some(language)
}

/// Installs a validated pack and rebuilds everything that shows game text.
fn install(app: &mut App, language: GameLanguage, pack: ValidatedLanguagePack) {
    let outcome = match app.data.as_mut() {
        Some(data) => data.engine.install_language_pack(pack),
        None => return,
    };
    match outcome {
        Ok(()) => {
            app.packs.set_language_failure(language, None);
            after_language_change(app);
        }
        Err(error) => failed(app, language, &error),
    }
}

/// Records why a pack failed, shows the matching notice, and falls back to English names.
fn failed(app: &mut App, language: GameLanguage, error: &AppError) {
    let kind = failure_kind(error);
    app.packs.set_language_failure(language, Some(kind));
    app.overlay.notice = Some(match kind {
        PackFailure::Mismatch => Notice::LanguagePackMismatch,
        PackFailure::Retryable | PackFailure::Permanent => Notice::LanguagePack,
    });
    revert_to_english(app);
}

/// Classifies a pack failure: another build's pack, a network or I/O failure worth retrying,
/// or a corrupt or missing pack that would fail again.
pub(super) fn failure_kind(error: &AppError) -> PackFailure {
    if matches!(error, AppError::PackMismatch(_)) {
        PackFailure::Mismatch
    } else if error.is_retryable() {
        PackFailure::Retryable
    } else {
        PackFailure::Permanent
    }
}

/// Drops an installed pack so game text is English again.
fn revert_to_english(app: &mut App) {
    let dropped = app
        .data
        .as_mut()
        .is_some_and(|data| data.engine.drop_language_pack().is_some());
    if dropped {
        after_language_change(app);
    }
}
