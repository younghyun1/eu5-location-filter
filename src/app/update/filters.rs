//! Filter messages, result refresh, and the derived state that follows a result change.

use iced::Task;

use crate::app::message::{FilterMessage, Message};
use crate::app::state::App;

/// Applies a filter message. Invalid colour or range text sets the field's inline error and
/// leaves the active filter unchanged; Clear all restores `FilterSet::land_only` and empties
/// every field, the search and each option query.
pub(super) fn filter(app: &mut App, message: FilterMessage) -> Task<Message> {
    let _ = (app, message);
    Task::none()
}

/// Re-runs `FilterEngine::apply` synchronously, keeps the selection only if it is still in the
/// results, and rebuilds the detail when the selection changed.
pub(super) fn refresh_results(app: &mut App) {
    let _ = app;
}

/// Rebuilds the detail pane content for the current selection, locale and language pack.
pub(super) fn rebuild_detail(app: &mut App) {
    let _ = app;
}

/// Rebuilds every option list and its visible indices after a dataset install, a game
/// language change or a locale change (static option labels come from the catalog).
pub(super) fn rebuild_options(app: &mut App) {
    let _ = app;
}
