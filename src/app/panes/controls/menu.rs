//! Popover frame shared by the column, language and appearance menus.

use iced::alignment::Vertical;
use iced::font::Weight;
use iced::widget::text::Wrapping;
use iced::widget::{Column, container, scrollable, text};
use iced::{Element, Length};

use super::buttons::icon_button;
use super::text::sans_font;
use crate::app::context::ViewContext;
use crate::app::layout::{MENU_MAX_HEIGHT, MENU_PADDING, MENU_WIDTH, inline_row};
use crate::app::message::{Message, OverlayMessage};
use crate::app::theme::{self, Icon, TEXT_BODY};

/// Popover frame shared by the column, language and appearance menus: `MENU_WIDTH` wide, at
/// most `MENU_MAX_HEIGHT` high, title row with a close button, scrolling content.
#[must_use]
pub(crate) fn menu_frame<'a>(
    ctx: ViewContext,
    title: &'a str,
    close_label: &'a str,
    content: Element<'a, Message>,
) -> Element<'a, Message> {
    let palette = ctx.palette;
    let heading = text(title)
        .size(TEXT_BODY)
        .font(sans_font(ctx, Weight::Semibold))
        .color(palette.ink)
        .width(Length::Fill)
        .align_x(ctx.direction.text_start())
        .wrapping(Wrapping::None);
    let close = icon_button(
        ctx,
        Icon::Close,
        close_label,
        Some(Message::Overlay(OverlayMessage::Close)),
    );
    let head = inline_row(ctx.direction, vec![heading.into(), close.into()])
        .spacing(MENU_PADDING)
        .align_y(Vertical::Center);
    let body = scrollable(content)
        .width(Length::Fill)
        .style(move |_theme, status| theme::scrollable(palette, status));
    let frame = Column::new()
        .push(head)
        .push(body)
        .spacing(MENU_PADDING)
        .width(Length::Fill);
    container(frame)
        .padding(MENU_PADDING)
        .width(MENU_WIDTH)
        .max_height(MENU_MAX_HEIGHT)
        .style(move |_theme| theme::menu(palette))
        .into()
}
