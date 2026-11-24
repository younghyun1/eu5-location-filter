//! Wide header: brand, search, status block, and the header buttons in inline order.

use iced::alignment::Vertical;
use iced::widget::text::Wrapping;
use iced::widget::{container, text};
use iced::{Element, Length, Padding};

use super::frame::framed;
use super::search::search_slot;
use super::status::{self, HeaderStatus};
use crate::app::context::ViewContext;
use crate::app::i18n::Text;
use crate::app::ids;
use crate::app::layout::{
    HEADER_BUTTON_WIDTH, HEADER_GAP, HEADER_HEIGHT, HEADER_PADDING_INLINE, HEADER_STATUS_WIDTH,
    SEARCH_MAX_WIDTH, inline_row,
};
use crate::app::message::{Message, OverlayMessage};
use crate::app::panes::controls::{mono_font, text_button};
use crate::app::state::Surface;
use crate::app::theme::{ButtonKind, TEXT_BRAND};
use crate::app::widget::FocusRing;

/// The header button opening `surface`; the opener of an open menu is drawn as primary.
fn opener<'a>(
    ctx: ViewContext,
    label: &'a str,
    surface: Surface,
    open: Option<Surface>,
    message: OverlayMessage,
) -> FocusRing<'a, Message> {
    let kind = if open == Some(surface) {
        ButtonKind::Primary
    } else {
        ButtonKind::Secondary
    };
    text_button(
        ctx,
        label,
        kind,
        HEADER_BUTTON_WIDTH,
        Some(Message::Overlay(message)),
    )
}

/// Wide header bar of height `HEADER_HEIGHT`. The brand slot is `brand_width` wide so the
/// search field starts where the grid starts; the search field takes the remaining width up to
/// `SEARCH_MAX_WIDTH`, and the status block and buttons follow it. Language and Appearance
/// exist only where the platform offers them.
#[must_use]
pub(super) fn bar<'a>(
    ctx: ViewContext,
    search: &'a str,
    status: HeaderStatus,
    brand_width: f32,
    open: Option<Surface>,
) -> Element<'a, Message> {
    let palette = ctx.palette;
    let brand = container(
        text(ctx.t(Text::AppTitle))
            .size(TEXT_BRAND)
            .font(mono_font(ctx, true))
            .color(palette.ink)
            .wrapping(Wrapping::None),
    )
    .width(brand_width)
    .align_x(ctx.direction.start())
    .center_y(Length::Fill)
    .clip(true);
    let search_field = container(
        container(search_slot(ctx, search))
            .width(Length::Fill)
            .max_width(SEARCH_MAX_WIDTH),
    )
    .width(Length::Fill)
    .align_x(ctx.direction.start())
    .center_y(Length::Fill);
    let status_block = container(status::block(ctx, status))
        .width(HEADER_STATUS_WIDTH)
        .center_y(Length::Fill);

    let mut items: Vec<Element<'a, Message>> =
        vec![brand.into(), search_field.into(), status_block.into()];
    if ctx.platform.has_desktop_controls() {
        items.push(
            opener(
                ctx,
                ctx.t(Text::HeaderLanguage),
                Surface::LanguageMenu,
                open,
                OverlayMessage::Toggle(Surface::LanguageMenu),
            )
            .id(ids::LANGUAGE_BUTTON)
            .into(),
        );
        items.push(
            opener(
                ctx,
                ctx.t(Text::HeaderAppearance),
                Surface::AppearanceMenu,
                open,
                OverlayMessage::Toggle(Surface::AppearanceMenu),
            )
            .id(ids::APPEARANCE_BUTTON)
            .into(),
        );
    }
    items.push(
        opener(
            ctx,
            ctx.t(Text::HeaderAbout),
            Surface::About,
            open,
            OverlayMessage::Open(Surface::About),
        )
        .id(ids::ABOUT_BUTTON)
        .into(),
    );

    let row = inline_row(ctx.direction, items)
        .spacing(HEADER_GAP)
        .align_y(Vertical::Center)
        .padding(Padding::from([0.0, HEADER_PADDING_INLINE]))
        .width(Length::Fill)
        .height(Length::Fill);
    framed(ctx, row.into(), HEADER_HEIGHT)
}
