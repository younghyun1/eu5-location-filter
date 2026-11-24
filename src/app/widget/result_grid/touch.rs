//! Touch input: a drag scrolls along the axis it locks to after `TOUCH_SLOP`, a press on a
//! header edge resizes that column, and a movement below the slop is a tap that selects a row
//! or sorts by a header. There is no fling.

use iced::Point;
use iced::advanced::Shell;
use iced::touch::{self, Finger};

use super::grid::ResultGrid;
use super::metrics::{Hit, Metrics};
use super::pointer::set_offsets;
use super::state::{Axis, GridDrag, GridState, TouchPan};
use crate::app::layout::TOUCH_SLOP;
use crate::app::message::SelectionSource;

/// Dispatches one touch event.
pub(super) fn handle<Message>(
    grid: &ResultGrid<'_, Message>,
    state: &mut GridState,
    metrics: &Metrics,
    event: &touch::Event,
    shell: &mut Shell<'_, Message>,
) {
    match *event {
        touch::Event::FingerPressed { id, position } => pressed(state, metrics, id, position, shell),
        touch::Event::FingerMoved { id, position } => moved(grid, state, metrics, id, position, shell),
        touch::Event::FingerLifted { id, position: _ } => lifted(grid, state, metrics, id, shell),
        touch::Event::FingerLost { id, position: _ } => {
            if state.touch.is_some_and(|pan| pan.finger == id) {
                state.touch = None;
            }
            if matches!(state.drag, Some(GridDrag::Resize { .. })) {
                state.drag = None;
                shell.request_redraw();
            }
        }
    }
}

/// A finger down inside the grid starts a resize on a header edge, else a tap or pan.
fn pressed<Message>(
    state: &mut GridState,
    metrics: &Metrics,
    finger: Finger,
    position: Point,
    shell: &mut Shell<'_, Message>,
) {
    if !metrics.frame.bounds.contains(position) {
        return;
    }
    if let Some(Hit::Edge(index)) = metrics.hit(position)
        && let Some(slot) = metrics.strip.slot(index)
    {
        state.drag = Some(GridDrag::Resize {
            column: slot.id,
            start_x: position.x,
            start_width: slot.width,
        });
        shell.request_redraw();
    } else {
        state.touch = Some(TouchPan {
            finger,
            start: position,
            last: position,
            axis: None,
        });
    }
    shell.capture_event();
}

/// Continues a resize, or scrolls along the locked axis with content following the finger.
fn moved<Message>(
    grid: &ResultGrid<'_, Message>,
    state: &mut GridState,
    metrics: &Metrics,
    finger: Finger,
    position: Point,
    shell: &mut Shell<'_, Message>,
) {
    if let Some(GridDrag::Resize { column, start_x, start_width }) = state.drag {
        if let Some(slot) = metrics.strip.position(column).and_then(|index| metrics.strip.slot(index)) {
            grid.request_width(shell, slot, start_width + metrics.frame.to_inline(position.x - start_x));
        }
        shell.capture_event();
        return;
    }
    let Some(mut pan) = state.touch.filter(|pan| pan.finger == finger) else {
        return;
    };
    if pan.axis.is_none() {
        let dx = position.x - pan.start.x;
        let dy = position.y - pan.start.y;
        if pan.start.distance(position) <= TOUCH_SLOP {
            shell.capture_event();
            return;
        }
        pan.axis = Some(if dx.abs() > dy.abs() { Axis::Horizontal } else { Axis::Vertical });
        // Apply the movement made inside the slop as well, so the content catches up.
        pan.last = pan.start;
    }
    let (offset_x, offset_y) = match pan.axis {
        Some(Axis::Horizontal) => (
            metrics.clamp_x(metrics.offset_x - metrics.frame.to_inline(position.x - pan.last.x)),
            metrics.offset_y,
        ),
        Some(Axis::Vertical) | None => (
            metrics.offset_x,
            metrics.clamp_y(metrics.offset_y - (position.y - pan.last.y)),
        ),
    };
    pan.last = position;
    state.touch = Some(pan);
    set_offsets(state, offset_x, offset_y, shell);
    shell.capture_event();
}

/// Ends a resize or pan; a finger that never left the slop taps what it pressed.
fn lifted<Message>(
    grid: &ResultGrid<'_, Message>,
    state: &mut GridState,
    metrics: &Metrics,
    finger: Finger,
    shell: &mut Shell<'_, Message>,
) {
    if matches!(state.drag, Some(GridDrag::Resize { .. })) {
        state.drag = None;
        shell.capture_event();
        shell.request_redraw();
        return;
    }
    let Some(pan) = state.touch.filter(|pan| pan.finger == finger) else {
        return;
    };
    state.touch = None;
    if pan.axis.is_none() {
        match metrics.hit(pan.start) {
            Some(Hit::Row(row)) => {
                grid.select_row(state, shell, row, SelectionSource::Pointer);
            }
            Some(Hit::Header(index)) => {
                if let Some(slot) = metrics.strip.slot(index) {
                    state.active_column = Some(slot.id);
                    grid.request_sort(shell, slot.id);
                }
            }
            Some(Hit::Edge(_) | Hit::VerticalLane | Hit::HorizontalLane) | None => {}
        }
        shell.request_redraw();
    }
    shell.capture_event();
}
