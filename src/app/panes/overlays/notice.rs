//! Notice bar under the header.

use iced::alignment::Vertical;
use iced::widget::container;
use iced::widget::space;
use iced::widget::text::Wrapping;
use iced::{Background, Element, Length, Padding, Theme};

use super::kit::sans;
use crate::app::context::ViewContext;
use crate::app::i18n::Text;
use crate::app::layout::{
    BUTTON_WIDTH_COMPACT, HEADER_PADDING_INLINE, MENU_PADDING, NARROW_PADDING_INLINE,
    NOTICE_HEIGHT, SELECTED_BAR_WIDTH, TOUCH_TARGET, inline_row,
};
use crate::app::message::{Message, OverlayMessage};
use crate::app::panes::controls::text_button;
use crate::app::state::Notice;
use crate::app::theme::{self, ButtonKind, TEXT_BODY};

/// Catalog text of a notice.
#[must_use]
pub(super) const fn notice_text(notice: Notice) -> Text {
    match notice {
        Notice::LanguagePack => Text::NoticeLanguagePack,
        Notice::LanguagePackMismatch => Text::NoticeLanguagePackMismatch,
        Notice::FontPack => Text::NoticeFontPack,
    }
}

/// Message, `danger` inline-start rule and Dismiss. The bar has a fixed height so that it never
/// asks its parent for more room than a notice needs: one `NOTICE_HEIGHT` line when wide, and
/// room for a message wrapped onto three lines beside the Dismiss button when narrow.
#[must_use]
pub(crate) fn notice_bar<'a>(ctx: ViewContext, notice: Notice) -> Element<'a, Message> {
    let palette = ctx.palette;
    let narrow = ctx.is_narrow();
    let wrapping = if narrow {
        Wrapping::WordOrGlyph
    } else {
        Wrapping::None
    };
    let message = sans(
        ctx,
        ctx.t(notice_text(notice)),
        TEXT_BODY,
        iced::font::Weight::Normal,
        palette.ink,
    )
    .width(Length::Fill)
    .wrapping(wrapping);
    let dismiss = text_button(
        ctx,
        ctx.t(Text::NoticeDismiss),
        ButtonKind::Secondary,
        BUTTON_WIDTH_COMPACT,
        Some(Message::Overlay(OverlayMessage::DismissNotice)),
    );
    let padding_inline = if narrow {
        NARROW_PADDING_INLINE
    } else {
        HEADER_PADDING_INLINE
    };
    let content = inline_row(ctx.direction, vec![message.into(), dismiss.into()])
        .spacing(MENU_PADDING)
        .padding(
            Padding::ZERO
                .horizontal(padding_inline)
                .vertical(MENU_PADDING / 2.0),
        )
        .align_y(Vertical::Center)
        .width(Length::Fill)
        .height(Length::Fill);
    let rule = container(space())
        .width(SELECTED_BAR_WIDTH)
        .height(Length::Fill)
        .style(move |_: &Theme| container::Style {
            background: Some(Background::Color(palette.danger)),
            ..container::Style::default()
        });
    let bar = inline_row(ctx.direction, vec![rule.into(), content.into()])
        .width(Length::Fill)
        .height(Length::Fill);
    let height = if narrow {
        TOUCH_TARGET + 3.0 * MENU_PADDING
    } else {
        NOTICE_HEIGHT
    };
    container(bar)
        .width(Length::Fill)
        .height(Length::Fixed(height))
        .style(move |_: &Theme| theme::notice(palette))
        .into()
}

#[cfg(test)]
mod tests {
    use super::{notice_bar, notice_text};
    use crate::app::i18n::Text;
    use crate::app::panes::overlays::kit::test_support::contexts;
    use crate::app::state::Notice;

    #[test]
    fn every_notice_has_its_own_message() {
        assert_eq!(notice_text(Notice::LanguagePack), Text::NoticeLanguagePack);
        assert_eq!(
            notice_text(Notice::LanguagePackMismatch),
            Text::NoticeLanguagePackMismatch
        );
        assert_eq!(notice_text(Notice::FontPack), Text::NoticeFontPack);
    }

    #[test]
    fn the_bar_builds_for_every_context() {
        for ctx in contexts() {
            for notice in [
                Notice::LanguagePack,
                Notice::LanguagePackMismatch,
                Notice::FontPack,
            ] {
                let _bar = notice_bar(ctx, notice);
            }
        }
    }
}
