//! `Widget` implementation of the splitter: a fixed-width, full-height hit strip that draws its
//! centred line and turns pointer, touch and arrow-key input into requested widths.

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::operation::Focusable;
use iced::advanced::widget::{self, Tree, tree};
use iced::advanced::{Clipboard, Renderer as _, Shell, Widget, renderer};
use iced::keyboard::{self, key::Named};
use iced::{Element, Event, Length, Rectangle, Size, mouse, touch};

use super::handle::Splitter;
use super::state::SplitterState;
use crate::app::layout::{SPLITTER_ACTIVE_LINE_WIDTH, SPLITTER_HIT_WIDTH, SPLITTER_LINE_WIDTH};

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
        tree: &Tree,
        renderer: &mut iced::Renderer,
        _theme: &iced::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<SplitterState>();
        let (width, color) = if state.is_active() {
            (SPLITTER_ACTIVE_LINE_WIDTH, self.palette.accent)
        } else {
            (SPLITTER_LINE_WIDTH, self.palette.line)
        };
        let bounds = layout.bounds();
        let line = Rectangle {
            x: bounds.center_x() - width / 2.0,
            width,
            ..bounds
        };
        renderer.fill_quad(
            renderer::Quad {
                bounds: line,
                ..renderer::Quad::default()
            },
            color,
        );
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
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<SplitterState>();
        let bounds = layout.bounds();
        let was_active = state.is_active();
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                match cursor.position_over(bounds) {
                    Some(position) if !shell.is_event_captured() => {
                        self.start_drag(state, position.x, None);
                        shell.capture_event();
                    }
                    Some(_) | None => state.unfocus(),
                }
            }
            Event::Touch(touch::Event::FingerPressed { id, .. }) => {
                // The window state moves the cursor to the touch location, so `cursor` also
                // reflects overlays above the splitter, which the raw position would not.
                match cursor.position_over(bounds) {
                    Some(position) if state.drag.is_none() && !shell.is_event_captured() => {
                        self.start_drag(state, position.x, Some(*id));
                        shell.capture_event();
                    }
                    Some(_) => {}
                    None => state.unfocus(),
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                state.hovered = cursor.is_over(bounds);
                if state.drag.is_some() && state.finger.is_none() {
                    self.drag_to(state, position.x, shell);
                }
            }
            Event::Touch(touch::Event::FingerMoved { id, position })
                if state.finger == Some(*id) =>
            {
                self.drag_to(state, position.x, shell);
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.drag.is_some() && state.finger.is_none() =>
            {
                state.drag = None;
                shell.capture_event();
            }
            Event::Touch(
                touch::Event::FingerLifted { id, .. } | touch::Event::FingerLost { id, .. },
            ) if state.finger == Some(*id) => {
                state.drag = None;
                state.finger = None;
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::CursorLeft) => state.hovered = false,
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if state.focused
                    && !shell.is_event_captured()
                    && !(modifiers.control() || modifiers.alt() || modifiers.logo()) =>
            {
                match key {
                    keyboard::Key::Named(Named::ArrowLeft) => self.step(-1.0, shell),
                    keyboard::Key::Named(Named::ArrowRight) => self.step(1.0, shell),
                    _ => {}
                }
            }
            _ => {}
        }
        if state.is_active() != was_active {
            shell.request_redraw();
        }
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
