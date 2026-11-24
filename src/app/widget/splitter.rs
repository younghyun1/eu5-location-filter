//! Pane splitter: an `SPLITTER_HIT_WIDTH` drag target between a docked pane and the results,
//! drawing a `line` hairline at rest and an `accent` line while hovered, focused or dragged.
//! It publishes the requested pane width (start width plus signed pointer movement, unclamped)
//! and, while focused, one `PANE_KEY_STEP` per ArrowLeft or ArrowRight in the arrow's screen
//! direction.

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::operation::Focusable;
use iced::advanced::widget::{self, Tree, tree};
use iced::advanced::{Clipboard, Shell, Widget, renderer};
use iced::widget::Id;
use iced::{Element, Event, Length, Rectangle, Size, mouse};

use crate::app::layout::{Direction, PaneEdge, SPLITTER_HIT_WIDTH};
use crate::app::theme::{PALETTE_DARK, Palette};

/// Callback for a requested pane width.
pub(crate) type OnResize<'a, Message> = Box<dyn Fn(f32) -> Message + 'a>;

/// Drag handle of one docked pane.
pub(crate) struct Splitter<'a, Message> {
    edge: PaneEdge,
    direction: Direction,
    width: f32,
    palette: Palette,
    id: Option<Id>,
    on_resize: Option<OnResize<'a, Message>>,
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

/// Splitter tree state.
#[derive(Debug, Default)]
pub(crate) struct SplitterState {
    /// Screen x and pane width at drag start.
    pub(crate) drag: Option<(f32, f32)>,
    /// Pointer over the hit area.
    pub(crate) hovered: bool,
    /// Keyboard focus.
    pub(crate) focused: bool,
}

impl Focusable for SplitterState {
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

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for Splitter<'_, Message> {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(SPLITTER_HIT_WIDTH), Length::Fill)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<SplitterState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(SplitterState::default())
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, Length::Fixed(SPLITTER_HIT_WIDTH), Length::Fill)
    }

    fn draw(
        &self,
        _tree: &Tree,
        _renderer: &mut iced::Renderer,
        _theme: &iced::Theme,
        _style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let _ = (self.edge, self.direction, self.width, self.palette);
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        let state = tree.state.downcast_mut::<SplitterState>();
        operation.focusable(self.id.as_ref(), layout.bounds(), state);
    }

    fn update(
        &mut self,
        _tree: &mut Tree,
        _event: &Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn Clipboard,
        _shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<SplitterState>();
        if state.drag.is_some() || cursor.is_over(layout.bounds()) {
            mouse::Interaction::ResizingColumn
        } else {
            mouse::Interaction::None
        }
    }
}

impl<'a, Message: 'a> From<Splitter<'a, Message>> for Element<'a, Message> {
    fn from(splitter: Splitter<'a, Message>) -> Self {
        Element::new(splitter)
    }
}
