//! The licence section of the About dialog: the summary lines, the show/hide button and the
//! expandable full texts.

use iced::widget::text::{Alignment, Wrapping};
use iced::widget::{Column, container};
use iced::{Element, Length, Theme};

use crate::app::context::ViewContext;
use crate::app::i18n::Text;
use crate::app::layout::{DIALOG_WIDTH, MENU_PADDING};
use crate::app::message::{Message, OverlayMessage};
use crate::app::panes::controls::text_button;
use crate::app::panes::overlays::kit::{mono, prose};
use crate::app::theme::{self, ButtonKind, LINE_HEIGHT_PROSE, TEXT_CAPTION, TEXT_META, TEXT_SMALL};

/// One licence text in a block of its own: left to right and in the mono face whatever the
/// interface language.
fn license_text<'a>(ctx: ViewContext, heading: Text, body: &'a str) -> Element<'a, Message> {
    let palette = ctx.palette;
    let text = mono(ctx, body, TEXT_META, false, palette.ink_muted)
        .line_height(LINE_HEIGHT_PROSE)
        .align_x(Alignment::Left)
        .width(Length::Fill)
        .wrapping(Wrapping::WordOrGlyph);
    let block = container(text)
        .padding(MENU_PADDING)
        .width(Length::Fill)
        .style(move |_: &Theme| theme::detail_block(palette));
    Column::new()
        .push(mono(ctx, ctx.t(heading), TEXT_CAPTION, true, palette.ink_faint).width(Length::Fill))
        .push(block)
        .spacing(MENU_PADDING / 2.0)
        .width(Length::Fill)
        .into()
}

/// The three licence lines, the show/hide button and, when expanded, the full texts.
pub(super) fn license_block<'a>(ctx: ViewContext, expanded: bool) -> Element<'a, Message> {
    let palette = ctx.palette;
    let mut column = Column::new().spacing(MENU_PADDING).width(Length::Fill);
    for line in [
        Text::AboutLicenseSource,
        Text::AboutLicenseUi,
        Text::AboutLicenseFonts,
    ] {
        column = column.push(prose(ctx, ctx.t(line), TEXT_SMALL, palette.ink_muted));
    }
    let label = if expanded {
        Text::AboutLicensesHide
    } else {
        Text::AboutLicensesShow
    };
    column = column.push(text_button(
        ctx,
        ctx.t(label),
        ButtonKind::Secondary,
        DIALOG_WIDTH / 2.0,
        Some(Message::Overlay(OverlayMessage::SetLicensesExpanded(
            !expanded,
        ))),
    ));
    if expanded {
        let texts = super::licenses::texts();
        column = column
            .push(license_text(
                ctx,
                Text::AboutLicenseSource,
                texts.source_mit.as_str(),
            ))
            .push(license_text(
                ctx,
                Text::AboutLicenseSource,
                texts.source_apache.as_str(),
            ))
            .push(license_text(ctx, Text::AboutLicenseUi, texts.iced.as_str()))
            .push(license_text(
                ctx,
                Text::AboutLicenseFonts,
                texts.fonts.as_str(),
            ));
    }
    column.into()
}
