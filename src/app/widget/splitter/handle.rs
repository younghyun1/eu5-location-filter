//! The splitter value: pane edge, direction, current width, palette, identifier and resize
//! callback, with the builders panes use and the helpers that turn movement into widths.

use iced::advanced::Shell;
use iced::touch;
use iced::widget::Id;

use super::state::SplitterState;
use crate::app::layout::{Direction, PANE_KEY_STEP, PaneEdge, key_step, splitter_width};
use crate::app::theme::{PALETTE_DARK, Palette};

/// Callback for a requested pane width.
pub(crate) type OnResize<'a, Message> = Box<dyn Fn(f32) -> Message + 'a>;

/// Drag handle of one docked pane.
pub(crate) struct Splitter<'a, Message> {
    pub(super) edge: PaneEdge,
    pub(super) direction: Direction,
    pub(super) width: f32,
    pub(super) palette: Palette,
    pub(super) id: Option<Id>,
    pub(super) on_resize: Option<OnResize<'a, Message>>,
}

impl<'a, Message> Splitter<'a, Message> {
    /// Handle for the pane on `edge` whose current width is `width`; `layout::splitter_width`
    /// and `layout::key_step` give the sign for the direction.
    #[must_use]
    pub(crate) fn new(edge: PaneEdge, direction: Direction, width: f32) -> Self {
        Self {
            edge,
            direction,
            width,
            palette: PALETTE_DARK,
            id: None,
            on_resize: None,
        }
    }

    /// Sets the identifier.
    #[must_use]
    pub(crate) fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the palette.
    #[must_use]
    pub(crate) fn palette(mut self, palette: Palette) -> Self {
        self.palette = palette;
        self
    }

    /// Publishes requested widths.
    #[must_use]
    pub(crate) fn on_resize(mut self, on_resize: impl Fn(f32) -> Message + 'a) -> Self {
        self.on_resize = Some(Box::new(on_resize));
        self
    }
}

impl<Message> Splitter<'_, Message> {
    /// Publishes the width for a pointer at screen x `x` during a drag; the requested width is
    /// always measured from the drag start, because `self.width` follows the clamped result.
    pub(super) fn drag_to(&self, state: &SplitterState, x: f32, shell: &mut Shell<'_, Message>) {
        if let (Some((start_x, start_width)), Some(on_resize)) = (state.drag, &self.on_resize) {
            shell.publish(on_resize(splitter_width(
                self.edge,
                self.direction,
                start_width,
                start_x,
                x,
            )));
        }
        shell.capture_event();
    }

    /// Starts a drag at screen x `x` and takes focus for the arrow keys.
    pub(super) fn start_drag(
        &self,
        state: &mut SplitterState,
        x: f32,
        finger: Option<touch::Finger>,
    ) {
        state.drag = Some((x, self.width));
        state.finger = finger;
        state.focused = true;
        state.focus_visible = false;
    }

    /// Publishes one `PANE_KEY_STEP` in the screen direction `sign` (-1 left, +1 right).
    pub(super) fn step(&self, sign: f32, shell: &mut Shell<'_, Message>) {
        if let Some(on_resize) = &self.on_resize {
            let delta = key_step(self.edge, self.direction, sign * PANE_KEY_STEP);
            shell.publish(on_resize(self.width + delta));
        }
        shell.capture_event();
    }
}
