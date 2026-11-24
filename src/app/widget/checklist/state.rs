//! Interaction state kept in the checklist's widget tree.

use iced::Point;
use iced::advanced::widget::operation::Focusable;
use iced::touch::Finger;

/// Checklist tree state.
#[derive(Debug, Default)]
pub(crate) struct ChecklistState {
    /// Vertical offset from the first row.
    pub(crate) offset: f32,
    /// Row under the pointer.
    pub(crate) hovered: Option<usize>,
    /// Keyboard-active row, outlined while focused.
    pub(crate) active: Option<usize>,
    /// Keyboard focus.
    pub(crate) focused: bool,
    /// Thumb drag; the pointer offset inside the thumb.
    pub(crate) thumb_grab: Option<f32>,
    /// Touch drag: finger, press position and previous position.
    pub(crate) touch: Option<(Finger, Point, Point)>,
    /// Row count seen in the previous update, so a shrinking filtered list clamps the offset
    /// and the active row.
    pub(crate) last_row_count: usize,
}

impl Focusable for ChecklistState {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn focus(&mut self) {
        self.focused = true;
    }

    fn unfocus(&mut self) {
        self.focused = false;
    }
}
