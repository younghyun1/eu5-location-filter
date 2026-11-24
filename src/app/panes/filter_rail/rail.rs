//! Rail frame: pinned header, collapsed strip, and the scrolling content.

use iced::alignment::Vertical;
use iced::font::Weight;
use iced::widget::text::Wrapping;
use iced::widget::{Column, container, rule, scrollable, space, text};
use iced::{Element, Length, Padding};

use super::sections;
use crate::app::context::ViewContext;
use crate::app::i18n::Text;
use crate::app::ids;
use crate::app::layout::{
    BORDER_WIDTH, BUTTON_WIDTH_COMPACT, RAIL_HEADER_HEIGHT, RAIL_PADDING_BLOCK,
    RAIL_PADDING_INLINE, RANGE_GAP, inline_row,
};
use crate::app::message::{FilterMessage, Message, OverlayMessage, PaneMessage};
use crate::app::panes::controls::{
    badge, counted, icon_button, sans_font, text_button, with_tooltip_owned,
};
use crate::app::state::TextFields;
use crate::app::theme::{self, ButtonKind, Icon, TEXT_BODY};
use crate::app::view_model::{FacetId, FacetOptions, FacetUi, active_filter_count};
use crate::filter::FilterSet;

/// Everything the rail shows.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RailModel<'a> {
    /// Active filters.
    pub(crate) filters: &'a FilterSet,
    /// Option lists; `None` before data loads, when groups show no options.
    pub(crate) options: Option<&'a FacetOptions>,
    /// Per-group state in `FacetId::ALL` order.
    pub(crate) facets: &'a [FacetUi; FacetId::COUNT],
    /// Field buffers and inline errors.
    pub(crate) fields: &'a TextFields,
}

/// The rail filling the width its parent gives it: the collapsed strip when `collapsed`, and a
/// Close button instead of the collapse button when `in_drawer`.
#[must_use]
pub(crate) fn view<'a>(
    ctx: ViewContext,
    rail: RailModel<'a>,
    collapsed: bool,
    in_drawer: bool,
) -> Element<'a, Message> {
    let palette = ctx.palette;
    let active = active_filter_count(rail.filters);
    let body: Element<'a, Message> = if collapsed && !in_drawer {
        strip(ctx, active)
    } else {
        let line = rule::horizontal(BORDER_WIDTH).style(move |_theme| theme::rule(palette, false));
        let content = scrollable(sections::content(ctx, rail))
            .id(ids::RAIL_SCROLL)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_theme, status| theme::scrollable(palette, status));
        Column::new()
            .push(header(ctx, active, in_drawer))
            .push(line)
            .push(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    };
    container(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme| theme::surface(palette))
        .into()
}

/// Badge with the active-filter count and its tooltip.
fn active_badge<'a>(ctx: ViewContext, active: usize) -> Element<'a, Message> {
    with_tooltip_owned(
        ctx,
        badge(ctx, active),
        counted(ctx, Text::RailActiveCount, active),
    )
}

/// Collapsed strip: the expand button and, with active filters, their badge below it.
fn strip<'a>(ctx: ViewContext, active: usize) -> Element<'a, Message> {
    let expand = icon_button(
        ctx,
        Icon::TowardEnd,
        ctx.t(Text::RailExpand),
        Some(Message::Pane(PaneMessage::SetRailCollapsed(false))),
    )
    .id(ids::RAIL_COLLAPSE);
    let mut column = Column::new()
        .push(expand)
        .spacing(RANGE_GAP)
        .padding(Padding::from([RAIL_PADDING_BLOCK, 0.0]))
        .width(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center);
    if active > 0 {
        column = column.push(active_badge(ctx, active));
    }
    column.into()
}

/// Pinned header: title, active-filter badge, Clear all, and the collapse button, or the Close
/// button in a drawer.
fn header<'a>(ctx: ViewContext, active: usize, in_drawer: bool) -> Element<'a, Message> {
    let palette = ctx.palette;
    let title = text(ctx.t(Text::RailTitle))
        .size(TEXT_BODY)
        .font(sans_font(ctx, Weight::Semibold))
        .color(palette.ink)
        .wrapping(Wrapping::None);
    let clear = text_button(
        ctx,
        ctx.t(Text::RailClearAll),
        ButtonKind::Secondary,
        BUTTON_WIDTH_COMPACT,
        Some(Message::Filter(FilterMessage::ClearAll)),
    )
    .id(ids::RAIL_CLEAR_ALL);
    let end: Element<'a, Message> = if in_drawer {
        let tip = ctx.t(Text::DrawerClose);
        icon_button(
            ctx,
            Icon::Close,
            tip,
            Some(Message::Overlay(OverlayMessage::CloseDrawer)),
        )
        .into()
    } else {
        let tip = ctx.t(Text::RailCollapse);
        icon_button(
            ctx,
            Icon::TowardStart,
            tip,
            Some(Message::Pane(PaneMessage::SetRailCollapsed(true))),
        )
        .id(ids::RAIL_COLLAPSE)
        .into()
    };
    let mut items: Vec<Element<'a, Message>> = vec![title.into()];
    if active > 0 {
        items.push(active_badge(ctx, active));
    }
    items.push(space::horizontal().into());
    items.push(clear.into());
    items.push(end);
    let row = inline_row(ctx.direction, items)
        .spacing(RANGE_GAP)
        .align_y(Vertical::Center)
        .padding(Padding::from([0.0, RAIL_PADDING_INLINE]))
        .width(Length::Fill);
    container(row)
        .width(Length::Fill)
        .center_y(RAIL_HEADER_HEIGHT)
        .into()
}
