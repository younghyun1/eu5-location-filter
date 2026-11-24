//! Bounds reporter. The browser build cannot receive input-method composition on its canvas,
//! so the host page places a real text field over the header search slot; this wrapper tells
//! the application where that slot is by publishing its window-space bounds whenever they
//! change. Layout, drawing and events pass through to the child unchanged.
//!
//! The check runs on every event the widget receives. The runtime sends
//! `window::Event::RedrawRequested` through `update` before each frame is drawn, so the first
//! layout and every later move or resize are published without waiting for input. The bounds are
//! those of the layout, which for the header are window coordinates because the header is not
//! inside a scrollable.

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{self, Tree, tree};
use iced::advanced::{Clipboard, Shell, Widget, overlay, renderer};
use iced::{Element, Event, Length, Rectangle, Size, Vector, mouse};

/// Wrapper that publishes its child's bounds through `on_bounds` when they change.
pub(crate) struct Anchor<'a, Message> {
    content: Element<'a, Message>,
    on_bounds: Box<dyn Fn(Rectangle) -> Message + 'a>,
}

/// Wraps `content`; `on_bounds` receives the bounds after the first layout and after every
/// later change of position or size, never more than once for equal bounds.
#[must_use]
pub(crate) fn anchor<'a, Message>(
    content: impl Into<Element<'a, Message>>,
    on_bounds: impl Fn(Rectangle) -> Message + 'a,
) -> Anchor<'a, Message> {
    Anchor {
        content: content.into(),
        on_bounds: Box::new(on_bounds),
    }
}

/// Last bounds published by one anchor.
#[derive(Debug, Default)]
pub(crate) struct AnchorState {
    /// Bounds sent with the most recent message.
    pub(crate) reported: Option<Rectangle>,
}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for Anchor<'_, Message> {
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<AnchorState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(AnchorState::default())
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
        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<AnchorState>();
        if state.reported != Some(bounds) {
            // Recorded before publishing, so the update pass the message triggers sees equal
            // bounds and publishes nothing further.
            state.reported = Some(bounds);
            shell.publish((self.on_bounds)(bounds));
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

impl<'a, Message: 'a> From<Anchor<'a, Message>> for Element<'a, Message> {
    fn from(anchor: Anchor<'a, Message>) -> Self {
        Element::new(anchor)
    }
}
