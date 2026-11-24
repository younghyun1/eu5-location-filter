//! Text helpers shared by the controls: fonts, captions, badges, counted strings and tooltips.

use iced::font::Weight;
use iced::widget::text::{IntoFragment, Wrapping};
use iced::widget::{container, text, tooltip};
use iced::{Element, Font, Padding};

use crate::app::context::ViewContext;
use crate::app::i18n::{Text, format, push_grouped, tp};
use crate::app::layout::{BORDER_WIDTH, BUTTON_PADDING_INLINE, MENU_OFFSET, MENU_PADDING};
use crate::app::message::Message;
use crate::app::theme::{self, MONO, MONO_MEDIUM, TEXT_CAPTION, TEXT_SMALL, sans_for};

/// Sans font for reading text in the interface locale. Locales with a script pack name the pack
/// family, so its glyphs win over the browser's fallback order.
#[must_use]
pub(crate) fn sans_font(ctx: ViewContext, weight: Weight) -> Font {
    sans_for(ctx.locale, weight)
}

/// Font for technical text (counts, captions, identifiers). Plex Mono has no glyphs outside
/// Latin, so locales with a script pack use that pack's face instead.
#[must_use]
pub(crate) fn mono_font(ctx: ViewContext, medium: bool) -> Font {
    match (ctx.locale.font_pack().is_some(), medium) {
        (true, true) => sans_for(ctx.locale, Weight::Medium),
        (true, false) => sans_for(ctx.locale, Weight::Normal),
        (false, true) => MONO_MEDIUM,
        (false, false) => MONO,
    }
}

/// Counted catalog text: the plural form of `key` for `count` with `{count}` filled by the
/// grouped number.
#[must_use]
pub(crate) fn counted(ctx: ViewContext, key: Text, count: usize) -> String {
    let count = u64::try_from(count).unwrap_or(u64::MAX);
    let mut digits = String::new();
    push_grouped(&mut digits, count);
    format(tp(ctx.locale, key, count), &[("count", digits.as_str())])
}

/// Pill badge with a count in Mono Medium `TEXT_CAPTION`.
#[must_use]
pub(crate) fn badge<'a>(ctx: ViewContext, count: usize) -> Element<'a, Message> {
    let palette = ctx.palette;
    let mut digits = String::new();
    push_grouped(&mut digits, u64::try_from(count).unwrap_or(u64::MAX));
    container(
        text(digits)
            .size(TEXT_CAPTION)
            .font(mono_font(ctx, true))
            .color(palette.accent_strong)
            .wrapping(Wrapping::None),
    )
    .padding(Padding::from([BORDER_WIDTH, BUTTON_PADDING_INLINE / 2.0]))
    .style(move |_theme| theme::badge(palette))
    .into()
}

/// Section or pane caption in Mono Medium `TEXT_CAPTION` `ink_faint`.
#[must_use]
pub(crate) fn caption<'a>(ctx: ViewContext, label: &'a str) -> Element<'a, Message> {
    text(label)
        .size(TEXT_CAPTION)
        .font(mono_font(ctx, true))
        .color(ctx.palette.ink_faint)
        .wrapping(Wrapping::None)
        .into()
}

/// Field or row label in Sans `TEXT_SMALL` `ink_muted`.
#[must_use]
pub(crate) fn small_label<'a>(ctx: ViewContext, label: &'a str) -> Element<'a, Message> {
    text(label)
        .size(TEXT_SMALL)
        .font(sans_font(ctx, Weight::Normal))
        .color(ctx.palette.ink_muted)
        .align_x(ctx.direction.text_start())
        .into()
}

/// Wraps any element in a tooltip with the shared style.
#[must_use]
pub(crate) fn with_tooltip<'a>(
    ctx: ViewContext,
    content: Element<'a, Message>,
    tip: &'a str,
) -> Element<'a, Message> {
    attach(ctx, content, tip)
}

/// Like `with_tooltip` for a tip built at view time, such as a counted string.
#[must_use]
pub(crate) fn with_tooltip_owned<'a>(
    ctx: ViewContext,
    content: Element<'a, Message>,
    tip: String,
) -> Element<'a, Message> {
    attach(ctx, content, tip)
}

/// Tooltip below `content`, offset by `MENU_OFFSET` and kept inside the viewport.
fn attach<'a>(
    ctx: ViewContext,
    content: Element<'a, Message>,
    tip: impl IntoFragment<'a>,
) -> Element<'a, Message> {
    let palette = ctx.palette;
    let body = text(tip)
        .size(TEXT_SMALL)
        .font(sans_font(ctx, Weight::Normal));
    tooltip(content, body, tooltip::Position::Bottom)
        .gap(MENU_OFFSET)
        .padding(MENU_PADDING)
        .snap_within_viewport(true)
        .style(move |_theme| theme::tooltip(palette))
        .into()
}
