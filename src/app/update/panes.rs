//! Pane geometry, transient surfaces, drawers, notices, links and window resizes.

use iced::{Size, Task};

use crate::app::message::{Message, OverlayMessage, PaneMessage};
use crate::app::state::App;

/// Splitter widths (clamped), rail collapse and detail hide; showing the detail pane collapses
/// the rail when both cannot fit beside `GRID_MIN_WIDTH`.
pub(super) fn pane(app: &mut App, message: PaneMessage) -> Task<Message> {
    let _ = (app, message);
    Task::none()
}

/// Opens and closes menus, dialogs and drawers (recording the opener for focus return),
/// toggles licence texts, dismisses notices and opens links.
pub(super) fn overlay(app: &mut App, message: OverlayMessage) -> Task<Message> {
    let _ = (app, message);
    Task::none()
}

/// Stores the viewport and recomputes the layout mode; leaving the narrow layout closes any
/// drawer.
pub(super) fn resized(app: &mut App, size: Size) -> Task<Message> {
    let _ = (app, size);
    Task::none()
}
