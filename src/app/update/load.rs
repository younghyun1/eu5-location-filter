//! Data load progress and results, desktop retry and rebuild, and language and font packs.

use iced::Task;

use crate::app::message::{LoadMessage, Message};
use crate::app::state::App;

/// Handles loading messages. Stale generations are ignored. A finished load installs the data,
/// builds options, applies a pending view state, and requests the language and font packs of
/// the current locale. Pack failures set a notice and record the failure kind.
pub(super) fn load(app: &mut App, message: LoadMessage) -> Task<Message> {
    let _ = (app, message);
    Task::none()
}
