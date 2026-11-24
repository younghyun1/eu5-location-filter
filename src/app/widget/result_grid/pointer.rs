//! Mouse input: hover, row and header clicks, header-edge resizing, thumb dragging,
//! middle-button panning and wheel scrolling. State changes without a message request a
//! redraw; handled events are captured.

use iced::advanced::Shell;
use iced::advanced::mouse::{Click, click};
use iced::mouse::{self, Button, Cursor, ScrollDelta};
use iced::Point;

use super::grid::ResultGrid;
use super::metrics::{Hit, Metrics};
use super::state::{Axis, GridDrag, GridState};
use crate::app::layout::SCROLLBAR_MIN_THUMB;
use crate::app::message::SelectionSource;
use crate::app::widget::window;

/// Dispatches one mouse event.
pub(super) fn handle<Message>(
    grid: &ResultGrid<'_, Message>,
    state: &mut GridState,
    metrics: &Metrics,
    event: &mouse::Event,
    cursor: Cursor,
    shell: &mut Shell<'_, Message>,
) {
    let bounds = metrics.frame.bounds;
    match event {
        mouse::Event::CursorMoved { position } => moved(grid, state, metrics, *position, cursor, shell),
        mouse::Event::ButtonPressed(Button::Left) => match cursor.position_over(bounds) {
            Some(point) => press(grid, state, metrics, point, shell),
            None => {
                if state.focused {
                    state.focused = false;
                    shell.request_redraw();
                }
            }
        },
        mouse::Event::ButtonPressed(Button::Middle) => {
            if let Some(point) = cursor.position_over(bounds) {
                state.drag = Some(GridDrag::Pan { last: point });
                shell.capture_event();
                shell.request_redraw();
            }
        }
        mouse::Event::ButtonReleased(_) => {
            if state.drag.take().is_some() {
                shell.capture_event();
                shell.request_redraw();
            }
        }
        mouse::Event::WheelScrolled { delta } => {
            if cursor.is_over(bounds) {
                wheel(state, metrics, *delta, cursor, shell);
            }
        }
        mouse::Event::CursorLeft => {
            if state.hovered_row.is_some() || state.hovered_edge.is_some() || state.hovered_lane.is_some() {
                state.hovered_row = None;
                state.hovered_edge = None;
                state.hovered_lane = None;
                shell.request_redraw();
            }
        }
        mouse::Event::CursorEntered | mouse::Event::ButtonPressed(_) => {}
    }
}

/// Left press inside the grid: focus, then sort, start a resize, select, or grab a thumb.
fn press<Message>(
    grid: &ResultGrid<'_, Message>,
    state: &mut GridState,
    metrics: &Metrics,
    point: Point,
    shell: &mut Shell<'_, Message>,
) {
    state.focused = true;
    match metrics.hit(point) {
        Some(Hit::Edge(index)) => {
            if let Some(slot) = metrics.strip.slot(index) {
                state.drag = Some(GridDrag::Resize {
                    column: slot.id,
                    start_x: point.x,
                    start_width: slot.width,
                });
            }
        }
        Some(Hit::Header(index)) => {
            if let Some(slot) = metrics.strip.slot(index) {
                state.active_column = Some(slot.id);
                grid.request_sort(shell, slot.id);
            }
        }
        Some(Hit::Row(row)) => {
            let click = Click::new(point, Button::Left, state.last_click);
            state.last_click = Some(click);
            let selected = grid.select_row(state, shell, row, SelectionSource::Pointer);
            if let (Some(id), click::Kind::Double) = (selected, click.kind()) {
                grid.request_detail(shell, id);
            }
        }
        Some(Hit::VerticalLane) => grab_vertical(state, metrics, point),
        Some(Hit::HorizontalLane) => grab_horizontal(state, metrics, point),
        None => {}
    }
    shell.capture_event();
    shell.request_redraw();
}

/// Starts a vertical thumb drag; a press on the track first centres the thumb on the pointer.
fn grab_vertical(state: &mut GridState, metrics: &Metrics, point: Point) {
    let Some(thumb) = metrics.vertical_thumb(metrics.offset_y) else {
        return;
    };
    let along = point.y - metrics.frame.vertical_lane.y;
    let grab = if along >= thumb.start && along <= thumb.start + thumb.length {
        along - thumb.start
    } else {
        state.offset_y = vertical_offset(metrics, along - thumb.length / 2.0);
        thumb.length / 2.0
    };
    state.drag = Some(GridDrag::VerticalThumb { grab });
}

/// Starts a horizontal thumb drag, measured from the inline start of the lane.
fn grab_horizontal(state: &mut GridState, metrics: &Metrics, point: Point) {
    let Some(thumb) = metrics.horizontal_thumb(metrics.offset_x) else {
        return;
    };
    let along = metrics.frame.lane_inline(point.x);
    let grab = if along >= thumb.start && along <= thumb.start + thumb.length {
        along - thumb.start
    } else {
        state.offset_x = horizontal_offset(metrics, along - thumb.length / 2.0);
        thumb.length / 2.0
    };
    state.drag = Some(GridDrag::HorizontalThumb { grab });
}

/// Vertical offset for a thumb start along the body height.
fn vertical_offset(metrics: &Metrics, start: f32) -> f32 {
    let track = metrics.frame.body.height;
    let offset = window::offset_for_thumb(start, metrics.content_height(), track, track, SCROLLBAR_MIN_THUMB);
    metrics.clamp_y(offset)
}

/// Horizontal offset for a thumb start along the body width.
fn horizontal_offset(metrics: &Metrics, start: f32) -> f32 {
    let track = metrics.frame.body.width;
    let content = metrics.strip.content_width();
    metrics.clamp_x(window::offset_for_thumb(start, content, track, track, SCROLLBAR_MIN_THUMB))
}

/// Pointer movement: continues a drag, or updates hover.
fn moved<Message>(
    grid: &ResultGrid<'_, Message>,
    state: &mut GridState,
    metrics: &Metrics,
    position: Point,
    cursor: Cursor,
    shell: &mut Shell<'_, Message>,
) {
    let frame = &metrics.frame;
    match state.drag {
        Some(GridDrag::Resize { column, start_x, start_width }) => {
            if let Some(slot) = metrics.strip.position(column).and_then(|index| metrics.strip.slot(index)) {
                grid.request_width(shell, slot, start_width + frame.to_inline(position.x - start_x));
            }
            shell.capture_event();
        }
        Some(GridDrag::VerticalThumb { grab }) => {
            let offset = vertical_offset(metrics, position.y - frame.vertical_lane.y - grab);
            set_offsets(state, metrics.offset_x, offset, shell);
            shell.capture_event();
        }
        Some(GridDrag::HorizontalThumb { grab }) => {
            let offset = horizontal_offset(metrics, frame.lane_inline(position.x) - grab);
            set_offsets(state, offset, metrics.offset_y, shell);
            shell.capture_event();
        }
        Some(GridDrag::Pan { last }) => {
            // Content follows the pointer physically in both directions.
            let offset = metrics.clamp_x(metrics.offset_x - frame.to_inline(position.x - last.x));
            state.drag = Some(GridDrag::Pan { last: position });
            set_offsets(state, offset, metrics.offset_y, shell);
            shell.capture_event();
        }
        None => hover(state, metrics, cursor, shell),
    }
}

/// Stores offsets and requests a redraw when they changed.
pub(super) fn set_offsets<Message>(state: &mut GridState, x: f32, y: f32, shell: &mut Shell<'_, Message>) {
    if x != state.offset_x || y != state.offset_y {
        state.offset_x = x;
        state.offset_y = y;
        shell.request_redraw();
    }
}

/// Recomputes the hovered row, resize edge and scrollbar lane under the cursor.
pub(super) fn hover<Message>(state: &mut GridState, metrics: &Metrics, cursor: Cursor, shell: &mut Shell<'_, Message>) {
    let hit = cursor.position_over(metrics.frame.bounds).and_then(|point| metrics.hit(point));
    let row = match hit {
        Some(Hit::Row(row)) => Some(row),
        _ => None,
    };
    let edge = match hit {
        Some(Hit::Edge(index)) => metrics.strip.slot(index).map(|slot| slot.id),
        _ => None,
    };
    let lane = match hit {
        Some(Hit::VerticalLane) => Some(Axis::Vertical),
        Some(Hit::HorizontalLane) => Some(Axis::Horizontal),
        _ => None,
    };
    if row != state.hovered_row || edge != state.hovered_edge || lane != state.hovered_lane {
        state.hovered_row = row;
        state.hovered_edge = edge;
        state.hovered_lane = lane;
        shell.request_redraw();
    }
}

/// Wheel scrolling: vertical rows, horizontal columns, Shift turning a vertical wheel into a
/// horizontal one. A positive delta moves the view up, and left on screen in both directions.
fn wheel<Message>(
    state: &mut GridState,
    metrics: &Metrics,
    delta: ScrollDelta,
    cursor: Cursor,
    shell: &mut Shell<'_, Message>,
) {
    let (dx, dy) = match delta {
        ScrollDelta::Lines { x, y } => (window::lines_to_pixels(x), window::lines_to_pixels(y)),
        ScrollDelta::Pixels { x, y } => (x, y),
    };
    let finite = |value: f32| if value.is_finite() { value } else { 0.0 };
    let (mut dx, mut dy) = (finite(dx), finite(dy));
    if state.modifiers.shift() && dx == 0.0 {
        dx = dy;
        dy = 0.0;
    }
    let mut moved = *metrics;
    moved.offset_x = metrics.clamp_x(metrics.offset_x - metrics.frame.to_inline(dx));
    moved.offset_y = metrics.clamp_y(metrics.offset_y - dy);
    set_offsets(state, moved.offset_x, moved.offset_y, shell);
    // Content moved under a still cursor, so the hovered row changed too.
    hover(state, &moved, cursor, shell);
    shell.capture_event();
}
