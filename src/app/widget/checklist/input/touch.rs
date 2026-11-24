//! Touch handling of the checklist: a drag scrolls the list, a short touch is a tap that toggles
//! the row under the press.

use iced::advanced::Shell;
use iced::{mouse, touch};

use super::super::geometry::{Geometry, is_tap};
use super::super::list::Checklist;
use super::super::model::ChecklistModel;
use super::super::state::ChecklistState;
use super::{refresh_hover, set_offset};

impl<Message, M: ChecklistModel> Checklist<'_, Message, M> {
    /// Touch: drag past `TOUCH_SLOP` scrolls, a shorter touch is a tap that toggles the row
    /// under the press.
    pub(super) fn handle_touch(
        &self,
        state: &mut ChecklistState,
        geometry: &Geometry,
        event: touch::Event,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) {
        match event {
            touch::Event::FingerPressed { id, position } => {
                let point = match cursor {
                    mouse::Cursor::Available(point) => point,
                    mouse::Cursor::Levitating(_) => return,
                    mouse::Cursor::Unavailable => position,
                };
                if !geometry.bounds.contains(point) {
                    if state.focused {
                        state.focused = false;
                        shell.request_redraw();
                    }
                    return;
                }
                state.touch = Some((id, position, position));
                state.touch_row = geometry.row_at(point, state.offset);
                state.touch_scrolling = false;
                if geometry.is_scrollable() {
                    shell.capture_event();
                }
            }
            touch::Event::FingerMoved { id, position } => {
                let Some((finger, start, last)) = state.touch else {
                    return;
                };
                if finger != id {
                    return;
                }
                if !state.touch_scrolling && !is_tap(start, position) {
                    state.touch_scrolling = true;
                }
                if state.touch_scrolling
                    && geometry.is_scrollable()
                    && set_offset(
                        state,
                        geometry.clamp(state.offset + (last.y - position.y)),
                        shell,
                    )
                {
                    refresh_hover(state, geometry, cursor, shell);
                }
                state.touch = Some((finger, start, position));
                if geometry.is_scrollable() {
                    shell.capture_event();
                }
            }
            touch::Event::FingerLifted { id, .. } => {
                let Some((finger, _, _)) = state.touch else {
                    return;
                };
                if finger != id {
                    return;
                }
                let tapped = state.touch_row.take().filter(|_| !state.touch_scrolling);
                state.touch = None;
                state.touch_scrolling = false;
                if let Some(row) = tapped.filter(|row| *row < geometry.count) {
                    state.active = Some(row);
                    self.toggle(row, shell);
                    shell.capture_event();
                    shell.request_redraw();
                }
            }
            touch::Event::FingerLost { id, .. } => {
                if state.touch.is_some_and(|(finger, _, _)| finger == id) {
                    state.touch = None;
                    state.touch_row = None;
                    state.touch_scrolling = false;
                }
            }
        }
    }
}
