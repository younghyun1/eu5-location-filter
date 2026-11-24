//! Event handling of the checklist: pointer hover, clicks and thumb drags, wheel and touch
//! scrolling, and keyboard navigation while focused. The only message is the toggle callback;
//! every other change stays in tree state and requests a redraw.
//!
//! Hit tests use the cursor rather than event positions, because an enclosing scrollable
//! translates the cursor into content space but passes touch positions through unchanged.
//! Wheel and touch scrolling capture the event only when the list itself scrolls, so a short
//! list or one at its edge leaves the rail scrollable around it in control, as the stock
//! scrollable does.

use iced::advanced::Shell;
use iced::keyboard::{self, Key, key::Named};
use iced::{Event, Point, Rectangle, mouse, touch};

use super::geometry::{Geometry, Nav, is_tap, navigate};
use super::list::Checklist;
use super::model::ChecklistModel;
use super::state::ChecklistState;
use crate::app::widget::window;

impl<Message, M: ChecklistModel> Checklist<'_, Message, M> {
    /// Handles one event for a list laid out in `bounds`.
    pub(super) fn handle_event(
        &self,
        state: &mut ChecklistState,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) {
        let count = self.model.row_count();
        let geometry = Geometry::new(bounds, self.row_height, count, self.direction.is_rtl());
        sync(state, &geometry, shell);

        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let (Some(grab), Some(point)) = (state.thumb_grab, cursor.position()) {
                    let offset = geometry.offset_for_thumb(point.y - bounds.y - grab);
                    set_offset(state, offset, shell);
                    shell.capture_event();
                }
                refresh_hover(state, &geometry, cursor, shell);
            }
            Event::Mouse(mouse::Event::CursorLeft) => {
                if state.hovered.take().is_some() || state.lane_hovered {
                    state.lane_hovered = false;
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                self.press(state, &geometry, cursor, shell);
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.thumb_grab.take().is_some() {
                    shell.capture_event();
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) if cursor.is_over(bounds) => {
                let dy = match *delta {
                    mouse::ScrollDelta::Lines { y, .. } => window::lines_to_pixels(y),
                    mouse::ScrollDelta::Pixels { y, .. } => y,
                };
                if set_offset(state, geometry.clamp(state.offset - dy), shell) {
                    shell.capture_event();
                    refresh_hover(state, &geometry, cursor, shell);
                }
            }
            Event::Touch(touch_event) => {
                self.handle_touch(state, &geometry, *touch_event, cursor, shell)
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if state.focused =>
            {
                if !modifiers.command() && !modifiers.alt() {
                    self.key(state, &geometry, key, cursor, shell);
                }
            }
            _ => {}
        }
    }

    /// Left press: focus follows the press, a row toggles, the lane starts a thumb drag.
    fn press(
        &self,
        state: &mut ChecklistState,
        geometry: &Geometry,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) {
        let Some(point) = cursor.position_over(geometry.bounds) else {
            if state.focused {
                state.focused = false;
                shell.request_redraw();
            }
            return;
        };
        state.focused = true;
        shell.request_redraw();
        shell.capture_event();
        if geometry.in_lane(point) {
            let Some(thumb) = geometry.thumb(state.offset) else {
                return;
            };
            let along = point.y - geometry.bounds.y;
            let grab = if (thumb.start..=thumb.start + thumb.length).contains(&along) {
                along - thumb.start
            } else {
                let grab = thumb.length / 2.0;
                let _ = set_offset(state, geometry.offset_for_thumb(along - grab), shell);
                grab
            };
            state.thumb_grab = Some(grab);
        } else if let Some(row) = geometry.row_at(point, state.offset) {
            state.active = Some(row);
            self.toggle(row, shell);
        }
    }

    /// Touch: drag past `TOUCH_SLOP` scrolls, a shorter touch is a tap that toggles the row
    /// under the press.
    fn handle_touch(
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

    /// Keys while focused; unhandled keys, Tab and Escape included, stay uncaptured.
    fn key(
        &self,
        state: &mut ChecklistState,
        geometry: &Geometry,
        key: &Key,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) {
        let nav = match key.as_ref() {
            Key::Named(Named::ArrowUp) => Some(Nav::Previous),
            Key::Named(Named::ArrowDown) => Some(Nav::Next),
            Key::Named(Named::PageUp) => Some(Nav::PageUp),
            Key::Named(Named::PageDown) => Some(Nav::PageDown),
            Key::Named(Named::Home) => Some(Nav::First),
            Key::Named(Named::End) => Some(Nav::Last),
            Key::Named(Named::Space) => None,
            _ => return,
        };
        // The row drawn as active: the stored one, or the first visible row after a Tab focus.
        let current = state
            .active
            .filter(|row| *row < geometry.count)
            .or_else(|| geometry.first_visible(state.offset));
        let target = match nav {
            Some(nav) => {
                current.and_then(|row| navigate(row, nav, geometry.count, geometry.page()))
            }
            None => current,
        };
        let Some(row) = target else {
            return;
        };
        state.active = Some(row);
        if set_offset(state, geometry.reveal(state.offset, row), shell) {
            refresh_hover(state, geometry, cursor, shell);
        }
        if nav.is_none() {
            self.toggle(row, shell);
        }
        shell.capture_event();
        shell.request_redraw();
    }

    /// Publishes the toggle of a row: its stable option index and the inverted checked state.
    fn toggle(&self, row: usize, shell: &mut Shell<'_, Message>) {
        if let Some(on_toggle) = &self.on_toggle {
            shell.publish(on_toggle(
                self.model.option(row),
                !self.model.is_checked(row),
            ));
        }
    }
}

/// Clamps state against the current rows and bounds: a filtered list can shrink between
/// frames, and the bounds can change with the layout.
fn sync<Message>(state: &mut ChecklistState, geometry: &Geometry, shell: &mut Shell<'_, Message>) {
    let count = geometry.count;
    if count != state.last_row_count {
        state.last_row_count = count;
        state.hovered = None;
        state.touch_row = None;
        state.active = match (state.active, count.checked_sub(1)) {
            (Some(row), Some(last)) => Some(row.min(last)),
            _ => None,
        };
        shell.request_redraw();
    }
    let _ = set_offset(state, geometry.clamp(state.offset), shell);
    if !geometry.is_scrollable() {
        state.thumb_grab = None;
    }
}

/// Stores a new offset; returns whether it changed and requests a redraw when it did.
fn set_offset<Message>(
    state: &mut ChecklistState,
    offset: f32,
    shell: &mut Shell<'_, Message>,
) -> bool {
    if (offset - state.offset).abs() <= f32::EPSILON {
        return false;
    }
    state.offset = offset;
    shell.request_redraw();
    true
}

/// Recomputes the hovered row and lane after the cursor or the offset moved.
fn refresh_hover<Message>(
    state: &mut ChecklistState,
    geometry: &Geometry,
    cursor: mouse::Cursor,
    shell: &mut Shell<'_, Message>,
) {
    let point: Option<Point> = cursor.position_over(geometry.bounds);
    let hovered = point.and_then(|point| geometry.row_at(point, state.offset));
    let lane_hovered = point.is_some_and(|point| geometry.in_lane(point));
    if hovered != state.hovered || lane_hovered != state.lane_hovered {
        state.hovered = hovered;
        state.lane_hovered = lane_hovered;
        shell.request_redraw();
    }
}
