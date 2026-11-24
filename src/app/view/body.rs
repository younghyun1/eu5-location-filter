//! Header and body: in the wide layout the header sits above a row holding, in inline order,
//! the rail, its splitter, the results (toolbar and grid), the detail splitter and the detail
//! pane, mirrored by `inline_row` for right-to-left locales; in the narrow layout the header
//! sits above the toolbar and the full-width grid, and the rail and detail pane become drawers
//! stacked by `surfaces`.

use iced::widget::{Column, container, space};
use iced::{Element, Length};

use crate::app::context::ViewContext;
use crate::app::i18n::Text;
use crate::app::ids;
use crate::app::layout::{
    self, GRID_ROW_HEIGHT, GRID_ROW_HEIGHT_TOUCH, LayoutMode, PaneEdge, PaneLayout, PaneSlot,
    RAIL_COLLAPSED_WIDTH, inline_row,
};
use crate::app::message::{GridMessage, Message, PaneMessage};
use crate::app::panes::filter_rail::{self, RailModel};
use crate::app::panes::header::{self, HeaderStatus};
use crate::app::panes::{detail, toolbar};
use crate::app::state::{App, Surface};
use crate::app::view_model::{RowContext, active_filter_count};
use crate::app::widget::{ResultGrid, Splitter};

/// Header above the body for the current layout mode.
#[must_use]
pub(super) fn body(app: &App, context: ViewContext) -> Element<'_, Message> {
    let panes = layout::resolve(app.viewport.width, &app.panes, app.layout_mode);
    let header_row = header::view(
        context,
        &app.filters.search,
        header_status(app),
        panes.brand_width,
        app.overlay.surface,
    );
    let column_menu_open = app.overlay.surface == Some(Surface::ColumnMenu);
    let content = match context.mode {
        LayoutMode::Wide => wide_row(app, context, &panes, column_menu_open),
        LayoutMode::Narrow => results(app, context, false, column_menu_open),
    };
    Column::new()
        .push(header_row)
        .push(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// The result grid over `DataState::results`, or an empty element before data loads.
#[must_use]
pub(super) fn grid(app: &App, context: ViewContext) -> Element<'_, Message> {
    let Some(data) = app.data.as_ref() else {
        return space().width(Length::Fill).height(Length::Fill).into();
    };
    let row_context = RowContext {
        labels: data.engine.labels(),
        locale: context.locale,
    };
    let narrow = context.is_narrow();
    let row_height = if narrow {
        GRID_ROW_HEIGHT_TOUCH
    } else {
        GRID_ROW_HEIGHT
    };
    ResultGrid::new(
        &data.results,
        row_context,
        &app.columns,
        app.sort,
        app.selection,
    )
    .id(ids::GRID)
    .direction(context.direction)
    .palette(context.palette)
    .row_height(row_height)
    .touch(narrow)
    .empty_text(
        context.t(Text::GridEmptyTitle),
        context.t(Text::GridEmptyBody),
    )
    .on_select(|id, source| Message::Grid(GridMessage::Select { id, source }))
    .on_sort(|column| Message::Grid(GridMessage::Sort(column)))
    .on_resize(|column, width| Message::Grid(GridMessage::ResizeColumn { column, width }))
    .on_show_detail(|id| Message::Grid(GridMessage::ShowDetail(id)))
    .into()
}

/// Everything the filter rail shows, borrowed from the application state; shared with the
/// narrow-layout filter drawer.
#[must_use]
pub(super) fn rail_model(app: &App) -> RailModel<'_> {
    RailModel {
        filters: &app.filters,
        options: app.data.as_ref().map(|data| &data.options),
        facets: &app.facets,
        fields: &app.fields,
    }
}

/// Result count, data build and active filter count for the header; the first two stay empty
/// until a dataset is installed.
fn header_status(app: &App) -> HeaderStatus {
    let active_filters = active_filter_count(&app.filters);
    match app.data.as_ref() {
        Some(data) => HeaderStatus {
            result_count: Some(data.results.len()),
            build_id: Some(data.dataset.stored.build_id),
            active_filters,
        },
        None => HeaderStatus {
            result_count: None,
            build_id: None,
            active_filters,
        },
    }
}

/// Wide body row from the resolved pane slots. Splitters exist only beside docked panes, and
/// the toolbar offers Show details whenever the detail pane is not docked, including when
/// `resolve` hid it to keep the grid at its minimum width.
///
/// The row always has five children with the results in the middle, so collapsing or hiding a
/// pane (or mirroring the row) never moves the results in the widget tree and the grid keeps
/// its scroll offsets; an absent pane or splitter is an empty element in its slot.
fn wide_row<'a>(
    app: &'a App,
    context: ViewContext,
    panes: &PaneLayout,
    column_menu_open: bool,
) -> Element<'a, Message> {
    let (rail, rail_splitter): (Element<'a, Message>, Element<'a, Message>) = match panes.rail {
        PaneSlot::Docked(width) => (
            fixed_width(
                filter_rail::view(context, rail_model(app), false, false),
                width,
            ),
            Splitter::new(PaneEdge::Start, context.direction, width)
                .id(ids::RAIL_SPLITTER)
                .palette(context.palette)
                .on_resize(|requested| Message::Pane(PaneMessage::ResizeRail(requested)))
                .into(),
        ),
        PaneSlot::Collapsed => (
            fixed_width(
                filter_rail::view(context, rail_model(app), true, false),
                RAIL_COLLAPSED_WIDTH,
            ),
            space().into(),
        ),
        PaneSlot::Hidden | PaneSlot::Drawer(_) => (space().into(), space().into()),
    };
    let (detail_splitter, detail_pane, detail_hidden): (
        Element<'a, Message>,
        Element<'a, Message>,
        bool,
    ) = match panes.detail {
        PaneSlot::Docked(width) => (
            Splitter::new(PaneEdge::End, context.direction, width)
                .id(ids::DETAIL_SPLITTER)
                .palette(context.palette)
                .on_resize(|requested| Message::Pane(PaneMessage::ResizeDetail(requested)))
                .into(),
            fixed_width(detail::view(context, app.detail.as_ref(), false), width),
            false,
        ),
        PaneSlot::Collapsed | PaneSlot::Hidden | PaneSlot::Drawer(_) => {
            (space().into(), space().into(), true)
        }
    };
    let children: Vec<Element<'a, Message>> = vec![
        rail,
        rail_splitter,
        results(app, context, detail_hidden, column_menu_open),
        detail_splitter,
        detail_pane,
    ];
    inline_row(context.direction, children)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Results region: toolbar above the grid, filling the space it is given. The narrow layout
/// never offers Show details, because its header has a Details button.
fn results(
    app: &App,
    context: ViewContext,
    detail_hidden: bool,
    column_menu_open: bool,
) -> Element<'_, Message> {
    Column::new()
        .push(toolbar::view(context, detail_hidden, column_menu_open))
        .push(grid(app, context))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// A docked pane at a fixed width and the full body height.
fn fixed_width(content: Element<'_, Message>, width: f32) -> Element<'_, Message> {
    container(content)
        .width(width)
        .height(Length::Fill)
        .into()
}
