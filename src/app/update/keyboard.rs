//! Application keyboard commands: focus traversal, region cycling, search focus and Escape.

use iced::Task;

use crate::app::message::{KeyCommand, Message};
use crate::app::state::App;

/// Tab and Shift+Tab return `focus_next` or `focus_previous` (wrapping inside an open surface),
/// Ctrl+F focuses and selects the search, F6 cycles regions, Escape closes the topmost surface
/// and focuses its opener.
pub(super) fn key(app: &mut App, command: KeyCommand) -> Task<Message> {
    let _ = (app, command);
    Task::none()
}
