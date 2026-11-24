//! Interaction state of one splitter: drag origin, touch finger, hover and focus.

use iced::advanced::widget::operation::Focusable;
use iced::touch;

/// Splitter tree state.
#[derive(Debug, Default)]
pub(crate) struct SplitterState {
    /// Screen x and pane width at drag start.
    pub(crate) drag: Option<(f32, f32)>,
    /// Finger driving a touch drag; `None` for a mouse drag or no drag.
    pub(crate) finger: Option<touch::Finger>,
    /// Pointer over the hit area.
    pub(crate) hovered: bool,
    /// Keyboard focus.
    pub(crate) focused: bool,
    /// Focus came from a focus operation, so the accent line is shown for it.
    pub(crate) focus_visible: bool,
}

impl SplitterState {
    /// Whether the line is drawn in its active style.
    pub(super) fn is_active(&self) -> bool {
        self.drag.is_some() || self.hovered || (self.focused && self.focus_visible)
    }
}

impl Focusable for SplitterState {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn focus(&mut self) {
        self.focused = true;
        self.focus_visible = true;
    }

    fn unfocus(&mut self) {
        self.focused = false;
        self.focus_visible = false;
    }
}
