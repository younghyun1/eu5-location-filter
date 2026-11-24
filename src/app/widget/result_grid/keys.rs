//! Keyboard input while the grid has focus: row selection with scroll-into-view, the active
//! column, Shift-arrow resizing, Enter to sort and Space to show the detail pane. Tab,
//! Escape and every other key stay uncaptured so the application handles them.

use iced::advanced::Shell;
use iced::keyboard::{self, Key, key::Named};

use super::grid::ResultGrid;
use super::metrics::Metrics;
use super::pointer::set_offsets;
use super::state::GridState;
use crate::app::layout::COLUMN_KEY_STEP;
use crate::app::message::SelectionSource;
use crate::app::widget::window;

/// Row movement requested by a key.
#[derive(Clone, Copy, Debug)]
enum Step {
    /// ArrowUp.
    Previous,
    /// ArrowDown.
    Next,
    /// PageUp: the visible row count minus one.
    PageUp,
    /// PageDown.
    PageDown,
    /// Home.
    First,
    /// End.
    Last,
}

/// Dispatches one keyboard event.
pub(super) fn handle<Message>(
    grid: &ResultGrid<'_, Message>,
    state: &mut GridState,
    metrics: &Metrics,
    event: &keyboard::Event,
    shell: &mut Shell<'_, Message>,
) {
    let (key, modifiers) = match event {
        keyboard::Event::ModifiersChanged(modifiers) => {
            state.modifiers = *modifiers;
            return;
        }
        keyboard::Event::KeyPressed { key, modifiers, .. } => (key, *modifiers),
        keyboard::Event::KeyReleased { .. } => return,
    };
    state.modifiers = modifiers;
    if !state.focused || modifiers.control() || modifiers.alt() || modifiers.logo() {
        return;
    }
    let handled = match key.as_ref() {
        Key::Named(Named::ArrowUp) => select(grid, state, metrics, Step::Previous, shell),
        Key::Named(Named::ArrowDown) => select(grid, state, metrics, Step::Next, shell),
        Key::Named(Named::PageUp) => select(grid, state, metrics, Step::PageUp, shell),
        Key::Named(Named::PageDown) => select(grid, state, metrics, Step::PageDown, shell),
        Key::Named(Named::Home) => select(grid, state, metrics, Step::First, shell),
        Key::Named(Named::End) => select(grid, state, metrics, Step::Last, shell),
        Key::Named(Named::ArrowLeft) if modifiers.shift() => {
            resize(grid, state, metrics, -1.0, shell)
        }
        Key::Named(Named::ArrowRight) if modifiers.shift() => {
            resize(grid, state, metrics, 1.0, shell)
        }
        Key::Named(Named::ArrowLeft) => move_column(grid, state, metrics, -1, shell),
        Key::Named(Named::ArrowRight) => move_column(grid, state, metrics, 1, shell),
        Key::Named(Named::Enter) => match grid.active_slot(state, &metrics.strip) {
            Some(slot) => {
                state.active_column = Some(slot.id);
                grid.request_sort(shell, slot.id);
                true
            }
            None => false,
        },
        Key::Named(Named::Space) | Key::Character(" ") => match grid.selected {
            Some(id) => {
                grid.request_detail(shell, id);
                true
            }
            None => false,
        },
        _ => false,
    };
    if handled {
        shell.capture_event();
    }
}

/// Moves the selection and scrolls it fully into view; without a selection, every step but
/// End starts from the first visible row.
fn select<Message>(
    grid: &ResultGrid<'_, Message>,
    state: &mut GridState,
    metrics: &Metrics,
    step: Step,
    shell: &mut Shell<'_, Message>,
) -> bool {
    let Some(last) = metrics.rows.checked_sub(1) else {
        return false;
    };
    let body = metrics.frame.body.height;
    let page = window::page_rows(body, metrics.row_height);
    let first_visible = metrics.visible_rows().start.min(last);
    let target = match (grid.selected_row(state), step) {
        (_, Step::First) => 0,
        (_, Step::Last) => last,
        (None, _) => first_visible,
        (Some(row), Step::Previous) => row.saturating_sub(1),
        (Some(row), Step::Next) => row.saturating_add(1).min(last),
        (Some(row), Step::PageUp) => row.saturating_sub(page),
        (Some(row), Step::PageDown) => row.saturating_add(page).min(last),
    };
    let offset = window::scroll_into_view(metrics.offset_y, target, metrics.row_height, body);
    set_offsets(state, metrics.offset_x, metrics.clamp_y(offset), shell);
    grid.select_row(state, shell, target, SelectionSource::Keyboard);
    shell.request_redraw();
    true
}

/// Moves the active column one step on screen and scrolls it into view. `screen_step` is -1
/// for ArrowLeft and 1 for ArrowRight; right to left reverses display order on screen.
fn move_column<Message>(
    grid: &ResultGrid<'_, Message>,
    state: &mut GridState,
    metrics: &Metrics,
    screen_step: i8,
    shell: &mut Shell<'_, Message>,
) -> bool {
    let strip = &metrics.strip;
    let Some(active) = grid.active_slot(state, strip) else {
        return false;
    };
    let toward_end = (screen_step > 0) != metrics.frame.rtl;
    let index = if toward_end {
        active
            .index
            .saturating_add(1)
            .min(strip.len().saturating_sub(1))
    } else {
        active.index.saturating_sub(1)
    };
    let Some(slot) = strip.slot(index) else {
        return false;
    };
    state.active_column = Some(slot.id);
    // The first column's span includes the inset, so scrolling to it reaches the content start.
    let start = if slot.index == 0 { 0.0 } else { slot.start };
    let width = slot.start + slot.width - start;
    let wanted = window::span_into_view(metrics.offset_x, start, width, metrics.frame.body.width);
    let offset = metrics.clamp_x(wanted);
    set_offsets(state, offset, metrics.offset_y, shell);
    shell.request_redraw();
    true
}

/// Moves the active column's inline-end edge by `COLUMN_KEY_STEP` in the arrow's screen
/// direction: Shift+ArrowRight widens in left to right layouts and narrows in right to left.
fn resize<Message>(
    grid: &ResultGrid<'_, Message>,
    state: &mut GridState,
    metrics: &Metrics,
    screen_sign: f32,
    shell: &mut Shell<'_, Message>,
) -> bool {
    let Some(slot) = grid.active_slot(state, &metrics.strip) else {
        return false;
    };
    state.active_column = Some(slot.id);
    let width = slot.width + metrics.frame.to_inline(screen_sign * COLUMN_KEY_STEP);
    grid.request_width(shell, slot, width);
    true
}
