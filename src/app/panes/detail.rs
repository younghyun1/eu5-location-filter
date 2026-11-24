//! Detail pane: pinned header with the hide button, then the colour block, name, key,
//! breadcrumb and field groups of the selected location, or the empty state.

use iced::alignment::{Horizontal, Vertical};
use iced::font::Weight;
use iced::widget::text::{Alignment, Wrapping};
use iced::widget::{Column, container, rule, scrollable, space};
use iced::{Color, Element, Length, Padding, Theme};

use crate::app::context::ViewContext;
use crate::app::i18n::Text;
use crate::app::ids;
use crate::app::layout::anchored_text;
use crate::app::layout::{
    BORDER_WIDTH, BUTTON_WIDTH, DETAIL_HEADER_HEIGHT, DETAIL_PADDING, DETAIL_SWATCH_HEIGHT,
    DIALOG_GAP, MENU_PADDING, inline_row,
};
use crate::app::message::{Message, OverlayMessage, PaneMessage};
use crate::app::panes::controls::{icon_button, text_button};
use crate::app::panes::overlays::kit::{mono, prose, sans, wrapped};
use crate::app::theme::{
    self, ButtonKind, Icon, TEXT_BODY, TEXT_CAPTION, TEXT_HEADING, TEXT_META, TEXT_TITLE,
};
use crate::app::view_model::LocationDetail;
use crate::model::MapColor;

mod rows;
#[cfg(test)]
mod tests;

use rows::{breadcrumb, group_view};

/// Colour of a map colour value packed as `0xRRGGBB`.
#[must_use]
fn swatch_color(color: MapColor) -> Color {
    let [_, red, green, blue] = color.0.to_be_bytes();
    Color::from_rgb8(red, green, blue)
}

/// The pane filling the width its parent gives it; a Close button replaces the hide button
/// when `in_drawer`.
#[must_use]
pub(crate) fn view<'a>(
    ctx: ViewContext,
    detail: Option<&'a LocationDetail>,
    in_drawer: bool,
) -> Element<'a, Message> {
    let palette = ctx.palette;
    let body: Element<'a, Message> = match detail {
        Some(detail) => scrollable(content(ctx, detail))
            .id(ids::DETAIL_SCROLL)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_: &Theme, status| theme::scrollable(palette, status))
            .into(),
        None => empty_state(ctx),
    };
    let divider =
        rule::horizontal(BORDER_WIDTH).style(move |_: &Theme| theme::rule(palette, false));
    let pane = Column::new()
        .push(header(ctx, in_drawer))
        .push(divider)
        .push(body)
        .width(Length::Fill)
        .height(Length::Fill);
    container(pane)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_: &Theme| theme::surface(palette))
        .into()
}

/// Pinned header: the caption and, at the inline end, Hide (wide) or Close (drawer).
fn header<'a>(ctx: ViewContext, in_drawer: bool) -> Element<'a, Message> {
    let title: Element<'a, Message> = anchored_text(
        mono(
            ctx,
            ctx.t(Text::DetailTitle),
            TEXT_CAPTION,
            true,
            ctx.palette.ink_faint,
        )
        .wrapping(Wrapping::None),
        ctx.direction.start(),
    );
    let control: Element<'a, Message> = if in_drawer {
        text_button(
            ctx,
            ctx.t(Text::DetailClose),
            ButtonKind::Secondary,
            BUTTON_WIDTH,
            Some(Message::Overlay(OverlayMessage::CloseDrawer)),
        )
        .id(ids::DETAIL_HIDE)
        .into()
    } else {
        // The pane is docked at the inline end, so the chevron points toward that edge.
        icon_button(
            ctx,
            Icon::TowardEnd,
            ctx.t(Text::DetailHide),
            Some(Message::Pane(PaneMessage::SetDetailHidden(true))),
        )
        .id(ids::DETAIL_HIDE)
        .into()
    };
    inline_row(ctx.direction, vec![title, control])
        .spacing(MENU_PADDING)
        .padding(Padding::ZERO.horizontal(DETAIL_PADDING))
        .align_y(Vertical::Center)
        .width(Length::Fill)
        .height(Length::Fixed(DETAIL_HEADER_HEIGHT))
        .into()
}

/// Shown while no location is selected.
fn empty_state<'a>(ctx: ViewContext) -> Element<'a, Message> {
    let palette = ctx.palette;
    let title = sans(
        ctx,
        ctx.t(Text::DetailEmptyTitle),
        TEXT_TITLE,
        Weight::Semibold,
        palette.ink,
    );
    // Natural width with default alignment; the column centres it. An explicit alignment
    // would displace a single right-to-left line.
    let body = prose(
        ctx,
        ctx.t(Text::DetailEmptyBody),
        TEXT_BODY,
        palette.ink_muted,
    )
    .align_x(Alignment::Center);
    let column = Column::new()
        .push(title)
        .push(body)
        .spacing(MENU_PADDING)
        .width(Length::Fill)
        .align_x(Horizontal::Center);
    container(column)
        .padding(DETAIL_PADDING)
        .center(Length::Fill)
        .into()
}

/// Colour block, name, key, breadcrumb and groups of one location.
fn content<'a>(ctx: ViewContext, detail: &'a LocationDetail) -> Element<'a, Message> {
    let palette = ctx.palette;
    let color = swatch_color(detail.color);
    let block = container(space())
        .width(Length::Fill)
        .height(Length::Fixed(DETAIL_SWATCH_HEIGHT))
        .style(move |_: &Theme| theme::swatch(palette, color));
    let codes = wrapped(
        ctx.direction,
        vec![
            mono(
                ctx,
                detail.hex.as_str(),
                TEXT_META,
                false,
                palette.ink_muted,
            )
            .into(),
            mono(
                ctx,
                detail.rgb.as_str(),
                TEXT_META,
                false,
                palette.ink_muted,
            )
            .into(),
        ],
        DIALOG_GAP,
        0.0,
    );
    let name = sans(
        ctx,
        detail.name.as_str(),
        TEXT_HEADING,
        Weight::Semibold,
        palette.ink,
    )
    .width(Length::Fill)
    .wrapping(Wrapping::WordOrGlyph);
    let key = mono(
        ctx,
        detail.key.as_str(),
        TEXT_META,
        false,
        palette.ink_muted,
    )
    .width(Length::Fill)
    .wrapping(Wrapping::WordOrGlyph);
    let mut column = Column::new()
        .push(block)
        .push(codes)
        .push(name)
        .push(key)
        .push(breadcrumb(ctx, &detail.breadcrumb))
        .spacing(MENU_PADDING)
        .padding(DETAIL_PADDING)
        .width(Length::Fill);
    for group in &detail.groups {
        column = column.push(group_view(ctx, group));
    }
    column.into()
}
