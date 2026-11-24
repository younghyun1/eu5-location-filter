//! `Widget` implementation of the checklist; same clipping and text rules as the result grid.
//! Drawing lives in `draw`, event handling in `input`, and the geometry both share in
//! `geometry`.

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Tree, tree};
use iced::advanced::{Clipboard, Shell, Widget};
use iced::{Event, Length, Rectangle, Size, mouse};

use super::geometry::{Geometry, list_height};
use super::list::Checklist;
use super::model::ChecklistModel;
use super::state::ChecklistState;

impl<Message, M: ChecklistModel> Widget<Message, iced::Theme, iced::Renderer>
    for Checklist<'_, Message, M>
{
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<ChecklistState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(ChecklistState::default())
    }

    /// Fills the width and asks for the rows it shows; an enclosing container may give less,
    /// and the list then scrolls within the height it received.
    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let height = list_height(
            self.model.row_count(),
            self.max_visible_rows,
            self.row_height,
            !self.empty_text.is_empty(),
        );
        layout::Node::new(limits.resolve(Length::Fill, Length::Shrink, Size::new(0.0, height)))
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
        let state = tree.state.downcast_ref::<ChecklistState>();
        self.draw_list(state, renderer, layout.bounds(), viewport);
    }

    /// Reports the list as focusable under its id, then each visible row label (or the empty
    /// text) as text with its row bounds, so simulator tests can find and click options by
    /// label.
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<ChecklistState>();
        operation.focusable(self.id.as_ref(), bounds, state);

        let count = self.model.row_count();
        let geometry = Geometry::new(bounds, self.row_height, count, self.direction.is_rtl());
        let offset = geometry.clamp(state.offset);
        if count == 0 {
            if !self.empty_text.is_empty() {
                operation.text(None, geometry.row_rect(0, 0.0), self.empty_text);
            }
            return;
        }
        let range = geometry.visible(offset, bounds);
        for row in range.start..range.end {
            let rect = geometry.row_rect(row, offset);
            if let Some(visible) = rect.intersection(&bounds) {
                operation.text(None, visible, self.model.label(row));
            }
        }
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
        let state = tree.state.downcast_mut::<ChecklistState>();
        self.handle_event(state, event, layout.bounds(), cursor, shell);
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<ChecklistState>();
        if state.thumb_grab.is_some() {
            return mouse::Interaction::Grabbing;
        }
        let bounds = layout.bounds();
        let Some(point) = cursor.position_over(bounds) else {
            return mouse::Interaction::None;
        };
        let geometry = Geometry::new(bounds, self.row_height, self.model.row_count(), self.direction.is_rtl());
        if geometry.row_at(point, geometry.clamp(state.offset)).is_some() {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::None
        }
    }
}
