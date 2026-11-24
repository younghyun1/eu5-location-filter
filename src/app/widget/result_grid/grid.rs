//! Builder of the result grid and its conversion into an element.

use iced::Element;
use iced::widget::Id;

use crate::app::layout::{Direction, GRID_ROW_HEIGHT};
use crate::app::message::SelectionSource;
use crate::app::theme::{PALETTE_DARK, Palette};
use crate::app::view_model::{ColumnId, Columns, RowContext, SortState};
use crate::model::LocationId;

/// Callback for a row selection.
pub(crate) type OnSelect<'a, Message> = Box<dyn Fn(LocationId, SelectionSource) -> Message + 'a>;
/// Callback for a sort request on a column.
pub(crate) type OnSort<'a, Message> = Box<dyn Fn(ColumnId) -> Message + 'a>;
/// Callback for a requested, unclamped column width.
pub(crate) type OnResize<'a, Message> = Box<dyn Fn(ColumnId, f32) -> Message + 'a>;
/// Callback for Space on a focused grid.
pub(crate) type OnShowDetail<'a, Message> = Box<dyn Fn(LocationId) -> Message + 'a>;

/// Virtualized, sortable, resizable result table.
///
/// Messages: `on_select` on click, tap, or arrow, page, Home and End keys (with the selection
/// scrolled into view); `on_sort` on a header click or Enter on the active column; `on_resize`
/// while dragging a header edge or on Shift with ArrowLeft or ArrowRight; `on_show_detail` on
/// Space. Tab is never captured.
pub(crate) struct ResultGrid<'a, Message> {
    /// Filtered, sorted result IDs; borrowed from the application state, never copied.
    pub(super) ids: &'a [LocationId],
    /// Text source and locale for cells.
    pub(super) context: RowContext<'a>,
    /// Widths and visibility.
    pub(super) columns: &'a Columns,
    /// Sorted column and direction, for header arrows and the initial active column.
    pub(super) sort: SortState,
    /// Selected location.
    pub(super) selected: Option<LocationId>,
    /// Column origin, scrollbar side and keyboard mapping.
    pub(super) direction: Direction,
    /// Colours for this frame.
    pub(super) palette: Palette,
    /// Body row height: `GRID_ROW_HEIGHT`, or `GRID_ROW_HEIGHT_TOUCH` when narrow.
    pub(super) row_height: f32,
    /// Narrow layout: thinner thumbs, touch slop handling.
    pub(super) touch: bool,
    /// Identifier for focus operations and simulator selectors.
    pub(super) id: Option<Id>,
    /// Empty-state title.
    pub(super) empty_title: &'a str,
    /// Empty-state body.
    pub(super) empty_body: &'a str,
    pub(super) on_select: Option<OnSelect<'a, Message>>,
    pub(super) on_sort: Option<OnSort<'a, Message>>,
    pub(super) on_resize: Option<OnResize<'a, Message>>,
    pub(super) on_show_detail: Option<OnShowDetail<'a, Message>>,
}

impl<'a, Message> ResultGrid<'a, Message> {
    /// A grid over borrowed results. Defaults: left to right, dark palette, wide row height,
    /// no callbacks.
    #[must_use]
    pub(crate) fn new(
        ids: &'a [LocationId],
        context: RowContext<'a>,
        columns: &'a Columns,
        sort: SortState,
        selected: Option<LocationId>,
    ) -> Self {
        Self {
            ids,
            context,
            columns,
            sort,
            selected,
            direction: Direction::Ltr,
            palette: PALETTE_DARK,
            row_height: GRID_ROW_HEIGHT,
            touch: false,
            id: None,
            empty_title: "",
            empty_body: "",
            on_select: None,
            on_sort: None,
            on_resize: None,
            on_show_detail: None,
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

    /// Sets the body row height.
    #[must_use]
    pub(crate) fn row_height(mut self, row_height: f32) -> Self {
        self.row_height = row_height;
        self
    }

    /// Enables touch metrics.
    #[must_use]
    pub(crate) fn touch(mut self, touch: bool) -> Self {
        self.touch = touch;
        self
    }

    /// Sets the empty-state texts.
    #[must_use]
    pub(crate) fn empty_text(mut self, title: &'a str, body: &'a str) -> Self {
        self.empty_title = title;
        self.empty_body = body;
        self
    }

    /// Publishes on row selection.
    #[must_use]
    pub(crate) fn on_select(
        mut self,
        on_select: impl Fn(LocationId, SelectionSource) -> Message + 'a,
    ) -> Self {
        self.on_select = Some(Box::new(on_select));
        self
    }

    /// Publishes on a sort request.
    #[must_use]
    pub(crate) fn on_sort(mut self, on_sort: impl Fn(ColumnId) -> Message + 'a) -> Self {
        self.on_sort = Some(Box::new(on_sort));
        self
    }

    /// Publishes requested column widths.
    #[must_use]
    pub(crate) fn on_resize(mut self, on_resize: impl Fn(ColumnId, f32) -> Message + 'a) -> Self {
        self.on_resize = Some(Box::new(on_resize));
        self
    }

    /// Publishes Space on a focused grid.
    #[must_use]
    pub(crate) fn on_show_detail(
        mut self,
        on_show_detail: impl Fn(LocationId) -> Message + 'a,
    ) -> Self {
        self.on_show_detail = Some(Box::new(on_show_detail));
        self
    }
}

impl<'a, Message: 'a> From<ResultGrid<'a, Message>> for Element<'a, Message> {
    fn from(grid: ResultGrid<'a, Message>) -> Self {
        Element::new(grid)
    }
}
