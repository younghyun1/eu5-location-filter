//! Keyboard focus for stock controls. iced 0.14 makes only text inputs focusable, so every
//! button, toggle and menu item is wrapped in a ring that reports focus through
//! `Operation::focusable` (so `focus_next` and `focus_previous` reach it), publishes its
//! activation message on Enter or Space while focused, and draws a `FOCUS_RING_WIDTH` `accent`
//! border with `RADIUS` corners around the child without changing layout.

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::operation::Focusable;
use iced::advanced::widget::{self, Tree, tree};
use iced::advanced::{Clipboard, Shell, Widget, overlay, renderer};
use iced::widget::Id;
use iced::{Element, Event, Length, Rectangle, Size, Vector, mouse};

use crate::app::theme::{PALETTE_DARK, Palette};

/// Focusable wrapper around one child element. Layout, drawing and events pass through to the
/// child unchanged; the ring adds focus, activation and the focus outline.
pub(crate) struct FocusRing<'a, Message> {
    content: Element<'a, Message>,
    id: Option<Id>,
    on_activate: Option<Message>,
    palette: Palette,
}

/// Wraps a control.
#[must_use]
pub(crate) fn focus_ring<'a, Message>(
    content: impl Into<Element<'a, Message>>,
) -> FocusRing<'a, Message> {
    FocusRing {
        content: content.into(),
        id: None,
        on_activate: None,
        palette: PALETTE_DARK,
    }
}

impl<Message> FocusRing<'_, Message> {
    /// Sets the identifier used by focus operations, focus return and simulator selectors.
    #[must_use]
    pub(crate) fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Message published on Enter or Space while focused; usually the child's press message.
    #[must_use]
    pub(crate) fn on_activate(mut self, message: Message) -> Self {
        self.on_activate = Some(message);
        self
    }

    /// Optional activation, for disabled controls.
    #[must_use]
    pub(crate) fn on_activate_maybe(mut self, message: Option<Message>) -> Self {
        self.on_activate = message;
        self
    }

    /// Sets the outline colour source.
    #[must_use]
    pub(crate) fn palette(mut self, palette: Palette) -> Self {
        self.palette = palette;
        self
    }
}

/// Focus flag of one ring.
#[derive(Debug, Default)]
pub(crate) struct RingState {
    /// Keyboard focus.
    pub(crate) focused: bool,
}

impl Focusable for RingState {
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

impl<Message: Clone> Widget<Message, iced::Theme, iced::Renderer> for FocusRing<'_, Message> {
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<RingState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(RingState::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        match tree.children.first_mut() {
            Some(child) => self.content.as_widget_mut().layout(child, renderer, limits),
            None => layout::Node::new(Size::ZERO),
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        if let Some(child) = tree.children.first() {
            self.content
                .as_widget()
                .draw(child, renderer, theme, style, layout, cursor, viewport);
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        let state = tree.state.downcast_mut::<RingState>();
        operation.focusable(self.id.as_ref(), layout.bounds(), state);
        if let Some(child) = tree.children.first_mut() {
            self.content
                .as_widget_mut()
                .operate(child, layout, renderer, operation);
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if let Some(child) = tree.children.first_mut() {
            self.content.as_widget_mut().update(
                child, event, layout, cursor, renderer, clipboard, shell, viewport,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        match tree.children.first() {
            Some(child) => self
                .content
                .as_widget()
                .mouse_interaction(child, layout, cursor, viewport, renderer),
            None => mouse::Interaction::None,
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        match tree.children.first_mut() {
            Some(child) => self
                .content
                .as_widget_mut()
                .overlay(child, layout, renderer, viewport, translation),
            None => None,
        }
    }
}

impl<'a, Message: Clone + 'a> From<FocusRing<'a, Message>> for Element<'a, Message> {
    fn from(ring: FocusRing<'a, Message>) -> Self {
        Element::new(ring)
    }
}
