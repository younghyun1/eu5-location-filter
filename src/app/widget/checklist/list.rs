//! Builder of the checklist and its conversion into an element.

use iced::widget::Id;
use iced::{Element, Font};

use super::model::ChecklistModel;
use crate::app::layout::{CHECKLIST_MAX_VISIBLE_ROWS, CHECKLIST_ROW_HEIGHT, Direction};
use crate::app::theme::{PALETTE_DARK, Palette, SANS};

/// Callback for a toggled option: stable option index and its new checked state.
pub(crate) type OnToggle<'a, Message> = Box<dyn Fn(u32, bool) -> Message + 'a>;

/// Virtualized checklist. Publishes `on_toggle` on a click, a tap below `TOUCH_SLOP`, or Space
/// on the active option. Its height is `min(rows, max_visible_rows) * row_height`, so it never
/// lays out rows it does not draw.
pub(crate) struct Checklist<'a, Message, M> {
    /// Row source, held by value so a view can build it on the fly.
    pub(super) model: M,
    /// Checkboxes and scrollbar side.
    pub(super) direction: Direction,
    /// Colours for this frame.
    pub(super) palette: Palette,
    /// Label font; `theme::sans_for(locale, ..)` so script packs win over fallback order.
    pub(super) font: Font,
    /// `CHECKLIST_ROW_HEIGHT`, or `CHECKLIST_ROW_HEIGHT_TOUCH` when narrow.
    pub(super) row_height: f32,
    /// Rows shown before the list scrolls.
    pub(super) max_visible_rows: usize,
    /// Narrow layout metrics.
    pub(super) touch: bool,
    /// Identifier for focus operations and simulator selectors.
    pub(super) id: Option<Id>,
    /// Text shown when the model has no rows.
    pub(super) empty_text: &'a str,
    pub(super) on_toggle: Option<OnToggle<'a, Message>>,
}

impl<'a, Message, M: ChecklistModel + 'a> Checklist<'a, Message, M> {
    /// A checklist over a row source with wide metrics and no callback.
    #[must_use]
    pub(crate) fn new(model: M) -> Self {
        Self {
            model,
            direction: Direction::Ltr,
            palette: PALETTE_DARK,
            font: SANS,
            row_height: CHECKLIST_ROW_HEIGHT,
            max_visible_rows: CHECKLIST_MAX_VISIBLE_ROWS,
            touch: false,
            id: None,
            empty_text: "",
            on_toggle: None,
        }
    }

    /// Sets the widget identifier.
    #[must_use]
    pub(crate) fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the layout direction.
    #[must_use]
    pub(crate) fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    /// Sets the palette.
    #[must_use]
    pub(crate) fn palette(mut self, palette: Palette) -> Self {
        self.palette = palette;
        self
    }

    /// Sets the label font.
    #[must_use]
    pub(crate) fn font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }

    /// Sets the row height.
    #[must_use]
    pub(crate) fn row_height(mut self, row_height: f32) -> Self {
        self.row_height = row_height;
        self
    }

    /// Sets the rows shown before scrolling; the column menu shows every column.
    #[must_use]
    pub(crate) fn max_visible_rows(mut self, rows: usize) -> Self {
        self.max_visible_rows = rows;
        self
    }

    /// Enables touch metrics.
    #[must_use]
    pub(crate) fn touch(mut self, touch: bool) -> Self {
        self.touch = touch;
        self
    }

    /// Sets the empty-state text.
    #[must_use]
    pub(crate) fn empty_text(mut self, text: &'a str) -> Self {
        self.empty_text = text;
        self
    }

    /// Publishes toggles.
    #[must_use]
    pub(crate) fn on_toggle(mut self, on_toggle: impl Fn(u32, bool) -> Message + 'a) -> Self {
        self.on_toggle = Some(Box::new(on_toggle));
        self
    }
}

impl<'a, Message: 'a, M: ChecklistModel + 'a> From<Checklist<'a, Message, M>>
    for Element<'a, Message>
{
    fn from(list: Checklist<'a, Message, M>) -> Self {
        Element::new(list)
    }
}
