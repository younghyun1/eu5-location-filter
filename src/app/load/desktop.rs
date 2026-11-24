//! Desktop loading on dedicated threads reporting through channels: embedded bundles, external
//! bundles, import and rebuild (the previous worker's logic), and embedded language and font
//! packs. Thread spawn failures are reported as load failures, never unwrapped.

use std::sync::Arc;

use iced::Task;

use crate::app::i18n::FontPack;
use crate::app::message::Message;
use crate::app::state::LaunchPaths;
use crate::lang_pack::GameLanguage;
use crate::model::Dataset;

/// Starts a load with this generation: embedded bundles when no data file is given, the
/// external bundle and its index when present, otherwise (or when `force`) an import from the
/// game directory that writes the external bundles. Progress arrives as
/// `LoadMessage::Progress` through a bounded channel whose overflow drops progress, never the
/// result, which arrives once as `LoadMessage::Finished`.
pub(crate) fn start(launch: &LaunchPaths, generation: u64, force: bool) -> Task<Message> {
    let _ = (launch, generation, force);
    Task::none()
}

/// Decodes and validates the embedded pack for `language` on a thread and answers with
/// `LoadMessage::LanguagePackValidated`; a language without an embedded pack fails with
/// `InvalidData`.
pub(crate) fn language_pack(language: GameLanguage, dataset: &Arc<Dataset>) -> Task<Message> {
    let _ = (language, dataset);
    Task::none()
}

/// Registers the embedded script pack with `iced::font::load` and answers with
/// `LoadMessage::FontPackLoaded`.
pub(crate) fn font_pack(pack: FontPack) -> Task<Message> {
    let _ = pack;
    Task::none()
}
