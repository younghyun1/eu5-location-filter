//! About dialog: title, version and data build, author, links, disclaimer, licences with the
//! expandable full texts, the keyboard table, and Close.

use iced::font::Weight;
use iced::widget::text::Wrapping;
use iced::widget::{Column, container, scrollable};
use iced::{Element, Length, Padding, Theme};

use crate::app::context::ViewContext;
use crate::app::i18n::{Text, format};
use crate::app::ids;
use crate::app::layout::{
    BUTTON_WIDTH, DIALOG_GAP, DIALOG_MAX_HEIGHT, DIALOG_PADDING, DIALOG_WIDTH, MENU_PADDING,
    SCROLLBAR_WIDTH, inline_row,
};
use crate::app::links::Link;
use crate::app::message::{Message, OverlayMessage};
use crate::app::panes::controls::text_button;
use crate::app::panes::overlays::kit::{mono, prose, sans, wrapped};
use crate::app::theme::{
    self, ButtonKind, TEXT_BODY, TEXT_CAPTION, TEXT_HEADING, TEXT_META, TEXT_SMALL,
};
use crate::model::EU5_GAME_VERSION;

mod keys;
mod license_block;
mod licenses;
#[cfg(test)]
mod tests;

use keys::{Cap, KeyRow, ROWS, command_key};
use license_block::license_block;

/// Version line: the version of this application.
#[must_use]
fn version_line(ctx: ViewContext) -> String {
    format(
        ctx.t(Text::AboutVersion),
        &[("version", env!("CARGO_PKG_VERSION"))],
    )
}

/// Data line: the game version and Steam build of the dataset, or the pending text before data
/// loads. The build is an identifier and is written without digit grouping.
#[must_use]
fn data_line(ctx: ViewContext, build_id: Option<u64>) -> String {
    match build_id {
        Some(build) => format(
            ctx.t(Text::AboutData),
            &[
                ("version", EU5_GAME_VERSION),
                ("build", build.to_string().as_str()),
            ],
        ),
        None => ctx.t(Text::HeaderBuildPending).to_owned(),
    }
}

/// Dialog card content; the caller centres it over a scrim. `build_id` is `None` before data
/// loads.
#[must_use]
pub(crate) fn view<'a>(
    ctx: ViewContext,
    build_id: Option<u64>,
    licenses_expanded: bool,
) -> Element<'a, Message> {
    let palette = ctx.palette;
    let body = Column::new()
        .push(header(ctx, build_id))
        .push(links(ctx))
        .push(prose(
            ctx,
            ctx.t(Text::AboutDisclaimer),
            TEXT_BODY,
            palette.ink_muted,
        ))
        .push(prose(
            ctx,
            ctx.t(Text::AboutTrademark),
            TEXT_SMALL,
            palette.ink_muted,
        ))
        .push(license_block(ctx, licenses_expanded))
        .push(keyboard_table(ctx))
        .spacing(DIALOG_GAP)
        .width(Length::Fill);
    // Room for the scrollbar, which floats over the content.
    let scroll = scrollable(container(body).padding(Padding::ZERO.right(SCROLLBAR_WIDTH)))
        .id(ids::ABOUT_SCROLL)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_: &Theme, status| theme::scrollable(palette, status));
    let close = text_button(
        ctx,
        ctx.t(Text::AboutClose),
        ButtonKind::Secondary,
        BUTTON_WIDTH,
        Some(Message::Overlay(OverlayMessage::Close)),
    );
    let close_row = container(close)
        .width(Length::Fill)
        .align_x(ctx.direction.end());
    let column = Column::new()
        .push(scroll)
        .push(close_row)
        .spacing(DIALOG_GAP)
        .width(Length::Fill)
        .height(Length::Fill);
    container(column)
        .padding(DIALOG_PADDING)
        .width(Length::Fill)
        .height(Length::Fill)
        .max_width(DIALOG_WIDTH)
        .max_height(DIALOG_MAX_HEIGHT)
        .style(move |_: &Theme| theme::card(palette))
        .into()
}

/// Title, version and data build, and the author.
fn header<'a>(ctx: ViewContext, build_id: Option<u64>) -> Element<'a, Message> {
    let palette = ctx.palette;
    Column::new()
        .push(
            sans(
                ctx,
                ctx.t(Text::AboutTitle),
                TEXT_HEADING,
                Weight::Semibold,
                palette.ink,
            )
            .width(Length::Fill),
        )
        .push(mono(ctx, version_line(ctx), TEXT_META, false, palette.ink_muted).width(Length::Fill))
        .push(
            mono(
                ctx,
                data_line(ctx, build_id),
                TEXT_META,
                false,
                palette.ink_muted,
            )
            .width(Length::Fill)
            .wrapping(Wrapping::WordOrGlyph),
        )
        .push(
            sans(
                ctx,
                ctx.t(Text::AboutAuthor),
                TEXT_BODY,
                Weight::Semibold,
                palette.ink,
            )
            .width(Length::Fill),
        )
        .spacing(MENU_PADDING / 2.0)
        .width(Length::Fill)
        .into()
}

/// The three link buttons; they wrap onto a second line in a narrow dialog.
fn links<'a>(ctx: ViewContext) -> Element<'a, Message> {
    let buttons = Link::ALL
        .into_iter()
        .map(|link| {
            text_button(
                ctx,
                ctx.t(link.title()),
                ButtonKind::Secondary,
                BUTTON_WIDTH,
                Some(Message::Overlay(OverlayMessage::OpenLink(link))),
            )
            .into()
        })
        .collect();
    wrapped(ctx.direction, buttons, MENU_PADDING, MENU_PADDING)
}

/// A key cap: the label in the mono face on a small raised block.
fn key_cap<'a>(ctx: ViewContext, label: &'a str) -> Element<'a, Message> {
    let palette = ctx.palette;
    container(mono(ctx, label, TEXT_CAPTION, true, palette.ink).wrapping(Wrapping::None))
        .padding(
            Padding::ZERO
                .horizontal(MENU_PADDING * 0.75)
                .vertical(MENU_PADDING / 4.0),
        )
        .style(move |_: &Theme| theme::detail_block(palette))
        .into()
}

/// The keys of one row: caps joined by `+` within a chord and `/` between alternatives.
fn key_caps<'a>(ctx: ViewContext, row: KeyRow) -> Element<'a, Message> {
    let palette = ctx.palette;
    let children: Vec<Element<'a, Message>> = row
        .caps
        .iter()
        .map(|cap| match *cap {
            Cap::Key(text) => key_cap(ctx, ctx.t(text)),
            Cap::Literal(label) => key_cap(ctx, label),
            Cap::Command => key_cap(ctx, ctx.t(command_key(ctx.platform))),
            Cap::Plus => mono(ctx, "+", TEXT_CAPTION, true, palette.ink_faint).into(),
            Cap::Or => mono(ctx, "/", TEXT_CAPTION, true, palette.ink_faint).into(),
        })
        .collect();
    wrapped(
        ctx.direction,
        children,
        MENU_PADDING / 2.0,
        MENU_PADDING / 2.0,
    )
}

/// The keyboard table: the keys on one side, what they do on the other.
fn keyboard_table<'a>(ctx: ViewContext) -> Element<'a, Message> {
    let palette = ctx.palette;
    let mut column = Column::new()
        .push(
            sans(
                ctx,
                ctx.t(Text::AboutKeysTitle),
                TEXT_BODY,
                Weight::Semibold,
                palette.ink,
            )
            .width(Length::Fill),
        )
        .spacing(MENU_PADDING)
        .width(Length::Fill);
    for row in ROWS {
        let caps = container(key_caps(ctx, row)).width(Length::FillPortion(2));
        let action = prose(ctx, ctx.t(row.action), TEXT_SMALL, palette.ink_muted)
            .width(Length::FillPortion(3));
        column = column.push(
            inline_row(ctx.direction, vec![caps.into(), action.into()])
                .spacing(DIALOG_GAP)
                .width(Length::Fill),
        );
    }
    column.into()
}
