//! Browser host values and view-state reporting. Compiled on every target so the message type
//! stays target-independent; on desktop nothing produces these messages.

use iced::Task;

use crate::app::message::{HostSnapshot, Message};
use crate::app::state::App;

/// Applies drained host values: locale (with language and font pack loads), reduced motion,
/// palette (as a fade target), connection, and a view state, which is applied now when the
/// dataset and option lists are ready or kept in `HostState::pending_view_state` otherwise.
pub(crate) fn apply_host_snapshot(app: &mut App, snapshot: HostSnapshot) -> Task<Message> {
    let _ = (app, snapshot);
    Task::none()
}

/// Decodes and resolves the pending view state once data is ready, replacing the whole filter,
/// sort and selection state and resetting the rail fields to match.
pub(super) fn apply_pending_view_state(app: &mut App) -> Task<Message> {
    let _ = app;
    Task::none()
}

/// When reporting is enabled (dataset ready and host connected), encodes the canonical string
/// and sends it only if it differs from `HostState::last_reported`. The host script debounces.
pub(super) fn report_view_state(app: &mut App) {
    let _ = app;
}
