//! Grid and column menu messages.

use iced::Task;

use super::filters::{rebuild_detail, refresh_results};
use super::panes::show_detail_pane;
use super::surfaces::open_drawer;
use crate::app::layout::{Drawer, LayoutMode};
use crate::app::message::{ColumnMessage, GridMessage, Message, SelectionSource};
use crate::app::state::App;
use crate::model::LocationId;

/// Selection, sort, column resize and Space. A pointer selection in the narrow layout also
/// opens the detail drawer; Space shows the detail pane or opens the drawer.
pub(super) fn grid(app: &mut App, message: GridMessage) -> Task<Message> {
    match message {
        GridMessage::Select { id, source } => {
            select(app, id);
            let tapped = source == SelectionSource::Pointer;
            if tapped && app.layout_mode == LayoutMode::Narrow && app.selection == Some(id) {
                return open_drawer(app, Drawer::Details);
            }
            Task::none()
        }
        GridMessage::Sort(column) => {
            app.sort = app.sort.toggled(column.sort_field());
            refresh_results(app);
            Task::none()
        }
        GridMessage::ResizeColumn { column, width } => {
            // `f32::clamp` passes NaN through, so a non-finite width is dropped here.
            if width.is_finite() {
                app.columns.set_width(column, width);
            }
            Task::none()
        }
        GridMessage::ShowDetail(id) => {
            select(app, id);
            if app.selection != Some(id) {
                return Task::none();
            }
            match app.layout_mode {
                LayoutMode::Narrow => open_drawer(app, Drawer::Details),
                LayoutMode::Wide => {
                    if app.panes.detail_hidden {
                        show_detail_pane(app);
                    }
                    Task::none()
                }
            }
        }
    }
}

/// Column visibility; hiding the last visible column is refused, and the column menu shows
/// its keep-one hint for that case, so the refusal needs no state here.
pub(super) fn column(app: &mut App, message: ColumnMessage) -> Task<Message> {
    let ColumnMessage::SetVisible { column, visible } = message;
    let _accepted = app.columns.set_visible(column, visible);
    Task::none()
}

/// Selects a location that is in the current results and rebuilds the detail when it changes.
fn select(app: &mut App, id: LocationId) {
    let present = app
        .data
        .as_ref()
        .is_some_and(|data| data.results.contains(&id));
    if !present || app.selection == Some(id) {
        return;
    }
    app.selection = Some(id);
    rebuild_detail(app);
}
