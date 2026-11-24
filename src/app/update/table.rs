//! Grid and column menu messages.

use iced::Task;

use crate::app::message::{ColumnMessage, GridMessage, Message};
use crate::app::state::App;

/// Selection, sort, column resize and Space. A pointer selection in the narrow layout also
/// opens the detail drawer; Space shows the detail pane or opens the drawer.
pub(super) fn grid(app: &mut App, message: GridMessage) -> Task<Message> {
    let _ = (app, message);
    Task::none()
}

/// Column visibility; hiding the last visible column is refused.
pub(super) fn column(app: &mut App, message: ColumnMessage) -> Task<Message> {
    let _ = (app, message);
    Task::none()
}
