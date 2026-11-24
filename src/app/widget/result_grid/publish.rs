//! Message publishing and selection bookkeeping shared by the pointer, touch and keyboard
//! handlers. Messages leave only through the four builder callbacks.

use iced::advanced::Shell;

use super::geometry::{Slot, Strip};
use super::grid::ResultGrid;
use super::metrics::Metrics;
use super::state::GridState;
use crate::app::layout::{COLUMN_MAX_WIDTH, COLUMN_MIN_WIDTH};
use crate::app::message::SelectionSource;
use crate::app::view_model::{ColumnId, column_for_sort};
use crate::app::widget::window;
use crate::model::LocationId;

impl<Message> ResultGrid<'_, Message> {
    /// Selects the row at an index and publishes it; remembers the selection so the echoed
    /// selection does not scroll again in `sync`.
    pub(super) fn select_row(
        &self,
        state: &mut GridState,
        shell: &mut Shell<'_, Message>,
        row: usize,
        source: SelectionSource,
    ) -> Option<LocationId> {
        let id = self.ids.get(row).copied()?;
        state.last_selected = Some(id);
        state.selected_index = Some(row);
        if let Some(on_select) = &self.on_select {
            shell.publish(on_select(id, source));
        }
        Some(id)
    }

    /// Publishes a sort request.
    pub(super) fn request_sort(&self, shell: &mut Shell<'_, Message>, column: ColumnId) {
        if let Some(on_sort) = &self.on_sort {
            shell.publish(on_sort(column));
        }
    }

    /// Publishes an unclamped width when it would change the clamped width of the column.
    pub(super) fn request_width(&self, shell: &mut Shell<'_, Message>, slot: Slot, width: f32) {
        if !width.is_finite() {
            return;
        }
        let clamped = width.max(COLUMN_MIN_WIDTH).min(COLUMN_MAX_WIDTH);
        if (clamped - slot.width).abs() < 0.01 {
            return;
        }
        if let Some(on_resize) = &self.on_resize {
            shell.publish(on_resize(slot.id, width));
        }
    }

    /// Publishes a detail request for a location.
    pub(super) fn request_detail(&self, shell: &mut Shell<'_, Message>, id: LocationId) {
        if let Some(on_show_detail) = &self.on_show_detail {
            shell.publish(on_show_detail(id));
        }
    }

    /// Row index of the current selection. The stored hint is used when it still points at the
    /// selected ID; otherwise one scan of the borrowed IDs finds it and refreshes the hint.
    pub(super) fn selected_row(&self, state: &mut GridState) -> Option<usize> {
        let selected = self.selected?;
        if let Some(index) = state.selected_index
            && self.ids.get(index) == Some(&selected)
        {
            return Some(index);
        }
        let index = self.ids.iter().position(|id| *id == selected);
        state.selected_index = index;
        index
    }

    /// Keyboard-active column: the stored one while visible, else the sorted column while
    /// visible, else the first visible column.
    pub(super) fn active_slot(&self, state: &GridState, strip: &Strip) -> Option<Slot> {
        let sorted = column_for_sort(self.sort.field);
        state
            .active_column
            .and_then(|id| strip.position(id))
            .or_else(|| strip.position(sorted))
            .and_then(|index| strip.slot(index))
            .or_else(|| strip.slot(0))
    }

    /// Reconciles tree state with this frame's data: clamps offsets to the content, drops a
    /// hover past the last row, and scrolls a selection made outside the grid into view once.
    pub(super) fn sync(&self, state: &mut GridState, metrics: &Metrics) {
        state.offset_x = metrics.offset_x;
        state.offset_y = metrics.offset_y;
        if self.selected != state.last_selected {
            state.last_selected = self.selected;
            state.selected_index = None;
            if let Some(row) = self.selected_row(state) {
                let body = metrics.frame.body.height;
                let wanted = window::scroll_into_view(metrics.offset_y, row, metrics.row_height, body);
                state.offset_y = metrics.clamp_y(wanted);
            }
        }
        self.drop_stale_hover(state, metrics);
    }

    /// Clears hover state that no longer points at existing rows or columns.
    fn drop_stale_hover(&self, state: &mut GridState, metrics: &Metrics) {
        if state.hovered_row.is_some_and(|row| row >= metrics.rows) {
            state.hovered_row = None;
        }
        if state.hovered_edge.is_some_and(|id| metrics.strip.position(id).is_none()) {
            state.hovered_edge = None;
        }
    }
}
