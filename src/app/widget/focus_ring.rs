//! Keyboard focus for stock controls. iced 0.14 makes only text inputs focusable, so every
//! button, toggle and menu item is wrapped in a ring that reports focus through
//! `Operation::focusable` (so `focus_next` and `focus_previous` reach it), publishes its
//! activation message on Enter or Space while focused, and draws a `FOCUS_RING_WIDTH` `accent`
//! border with `RADIUS` corners around the child without changing layout.
//!
//! A pointer press focuses the ring the way it focuses a text input, so Tab continues from the
//! pressed control, and a press elsewhere unfocuses it. The outline follows the site's
//! focus-visible rule: it is drawn only when focus arrived through a focus operation (Tab, F6,
//! focus return), not after a pointer press.

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::operation::Focusable;
use iced::advanced::widget::{self, Tree, tree};
use iced::advanced::{Clipboard, Renderer as _, Shell, Widget, overlay, renderer};
use iced::keyboard::{self, key::Named};
use iced::widget::Id;
use iced::{Border, Color, Element, Event, Length, Rectangle, Size, Vector, mouse, touch};

use crate::app::layout::{FOCUS_RING_WIDTH, RADIUS};
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
    /// Focus came from a focus operation rather than a pointer press, so the outline is drawn.
    pub(crate) visible: bool,
}

impl Focusable for RingState {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn focus(&mut self) {
        self.focused = true;
        self.visible = true;
    }

    fn unfocus(&mut self) {
        self.focused = false;
        self.visible = false;
    }
}

/// Whether a key press activates a focused control: Enter or Space without Ctrl, Alt or the
/// logo key, and not an auto-repeat, so holding the key does not fire the action repeatedly.
fn is_activation(pressed: &keyboard::Key, modifiers: keyboard::Modifiers, repeat: bool) -> bool {
    let plain = !(modifiers.control() || modifiers.alt() || modifiers.logo());
    plain && !repeat && matches!(pressed, keyboard::Key::Named(Named::Enter | Named::Space))
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
        let state = tree.state.downcast_ref::<RingState>();
        if state.focused && state.visible {
            let bounds = layout.bounds();
            // A layer of its own keeps the outline above the child's text, which the renderer
            // otherwise draws after every quad of the same layer.
            renderer.with_layer(bounds, |renderer| {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds,
                        border: Border {
                            color: self.palette.accent,
                            width: FOCUS_RING_WIDTH,
                            radius: RADIUS.into(),
                        },
                        ..renderer::Quad::default()
                    },
                    Color::TRANSPARENT,
                );
            });
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
        let state = tree.state.downcast_mut::<RingState>();
        match event {
            // Focus follows the press whether or not the child captured it, as text inputs do.
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let pressed = cursor.is_over(layout.bounds());
                if pressed != state.focused || (pressed && state.visible) {
                    state.focused = pressed;
                    state.visible = false;
                    shell.request_redraw();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                modifiers,
                repeat,
                ..
            }) if state.focused
                && !shell.is_event_captured()
                && is_activation(key, *modifiers, *repeat) =>
            {
                if let Some(message) = self.on_activate.as_ref() {
                    shell.publish(message.clone());
                    shell.capture_event();
                }
            }
            _ => {}
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
