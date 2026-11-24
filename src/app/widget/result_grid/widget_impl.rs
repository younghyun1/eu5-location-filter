//! `Widget` implementation of the result grid. Drawing clips every column with `with_layer`
//! (tiny-skia ignores per-text clip bounds), draws text with infinite bounds and
//! `text::Alignment::Default` for start-aligned cells so right-to-left labels lay out, and uses
//! `Right` only for text without a right-to-left run. Each pass lives in its own module; this
//! file only builds the shared metrics and delegates.

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Renderer as _};
use iced::advanced::widget::{self, Tree, tree};
use iced::advanced::{Clipboard, Shell, Widget};
use iced::{Event, Length, Point, Rectangle, Size, mouse};

use super::grid::ResultGrid;
use super::metrics::{Hit, Metrics};
use super::state::{GridDrag, GridState};
use super::{draw_body, draw_chrome, draw_header, keys, pointer, report, touch};

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for ResultGrid<'_, Message> {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<GridState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(GridState::default())
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        _renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let node = layout::atomic(limits, Length::Fill, Length::Fill);
        // Results, widths and selection may have changed since the last frame; layout runs
        // after every rebuild, so this is where tree state catches up with them.
        let state = tree.state.downcast_mut::<GridState>();
        let metrics = Metrics::new(self, Rectangle::new(Point::ORIGIN, node.size()), state);
        self.sync(state, &metrics);
        node
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        _theme: &iced::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<GridState>();
        let bounds = layout.bounds();
        let Some(clip) = bounds.intersection(viewport) else {
            return;
        };
        let metrics = Metrics::new(self, bounds, state);
        draw_body::grounds(self, state, &metrics, renderer, clip);
        // The buffer is only borrowed here and in operate, which never overlap; skipping the
        // cells for one frame is the fallback if that ever changes.
        if let Ok(mut buffer) = state.buffer.try_borrow_mut() {
            draw_body::cells(self, &metrics, renderer, clip, &mut *buffer);
        }
        draw_header::labels(self, &metrics, renderer, clip);
        renderer.with_layer(clip, |renderer| {
            draw_chrome::chrome(self, state, &metrics, renderer, clip);
        });
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        let state = tree.state.downcast_mut::<GridState>();
        let bounds = layout.bounds();
        operation.focusable(self.id.as_ref(), bounds, state);
        let metrics = Metrics::new(self, bounds, state);
        report::visible_text(self, state, &metrics, operation);
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
        let state = tree.state.downcast_mut::<GridState>();
        let metrics = Metrics::new(self, layout.bounds(), state);
        state.offset_x = metrics.offset_x;
        state.offset_y = metrics.offset_y;
        match event {
            Event::Mouse(event) => pointer::handle(self, state, &metrics, event, cursor, shell),
            Event::Touch(event) => touch::handle(self, state, &metrics, event, shell),
            Event::Keyboard(event) => keys::handle(self, state, &metrics, event, shell),
            Event::Window(_) | Event::InputMethod(_) => {}
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
        let state = tree.state.downcast_ref::<GridState>();
        match state.drag {
            Some(GridDrag::Resize { .. }) => return mouse::Interaction::ResizingHorizontally,
            Some(GridDrag::Pan { .. }) => return mouse::Interaction::Grabbing,
            Some(GridDrag::VerticalThumb { .. } | GridDrag::HorizontalThumb { .. }) => {
                return mouse::Interaction::Idle;
            }
            None => {}
        }
        let bounds = layout.bounds();
        let Some(point) = cursor.position_over(bounds) else {
            return mouse::Interaction::None;
        };
        match Metrics::new(self, bounds, state).hit(point) {
            Some(Hit::Edge(_)) => mouse::Interaction::ResizingHorizontally,
            Some(Hit::Header(_) | Hit::Row(_)) => mouse::Interaction::Pointer,
            Some(Hit::VerticalLane | Hit::HorizontalLane) => mouse::Interaction::Idle,
            None => mouse::Interaction::None,
        }
    }
}
