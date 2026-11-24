//! Small builders shared by the overlays, dialogs, menus and the detail pane: localized text in
//! the right face, wrapping rows that follow the layout direction, the centred card frame, and
//! the marker that scopes keyboard focus to the open surface.

use iced::font::Weight;
use iced::widget::text::{IntoFragment, Wrapping};
use iced::widget::{Row, Text, container, text};
use iced::{Color, Element, Font, Length, Theme};

use crate::app::context::ViewContext;
use crate::app::ids;
use crate::app::layout::{DIALOG_MARGIN, DIALOG_PADDING, Direction};
use crate::app::message::Message;
use crate::app::theme::{self, LINE_HEIGHT, LINE_HEIGHT_PROSE, MONO, MONO_MEDIUM, sans_for};

/// Single-line localized text in the locale's sans face, aligned to the inline start. The
/// script pack family is named explicitly because cosmic-text has no locale-aware fallback in
/// the browser.
#[must_use]
pub(crate) fn sans<'a>(
    ctx: ViewContext,
    content: impl IntoFragment<'a>,
    size: f32,
    weight: Weight,
    color: Color,
) -> Text<'a> {
    text(content)
        .size(size)
        .font(sans_for(ctx.locale, weight))
        .line_height(LINE_HEIGHT)
        .color(color)
        .align_x(ctx.direction.text_start())
}

/// Wrapping localized text filling the width it is given, with the prose line height.
#[must_use]
pub(crate) fn prose<'a>(
    ctx: ViewContext,
    content: impl IntoFragment<'a>,
    size: f32,
    color: Color,
) -> Text<'a> {
    sans(ctx, content, size, Weight::Normal, color)
        .line_height(LINE_HEIGHT_PROSE)
        .width(Length::Fill)
        .wrapping(Wrapping::WordOrGlyph)
}

/// Face for technical text such as captions, identifiers and licence texts. Plex Mono has no
/// glyphs outside Latin, so locales with a script pack use that pack's face instead.
#[must_use]
pub(crate) fn mono_face(ctx: ViewContext, medium: bool) -> Font {
    match (ctx.locale.font_pack().is_some(), medium) {
        (true, true) => sans_for(ctx.locale, Weight::Medium),
        (true, false) => sans_for(ctx.locale, Weight::Normal),
        (false, true) => MONO_MEDIUM,
        (false, false) => MONO,
    }
}

/// Technical text in the mono face, aligned to the inline start.
#[must_use]
pub(crate) fn mono<'a>(
    ctx: ViewContext,
    content: impl IntoFragment<'a>,
    size: f32,
    medium: bool,
    color: Color,
) -> Text<'a> {
    text(content)
        .size(size)
        .font(mono_face(ctx, medium))
        .line_height(LINE_HEIGHT)
        .color(color)
        .align_x(ctx.direction.text_start())
}

/// Children in inline order laid out in a row that wraps onto further lines, mirrored for
/// right-to-left layouts: the children are reversed and the lines align to the inline start.
#[must_use]
pub(crate) fn wrapped<'a>(
    direction: Direction,
    mut children: Vec<Element<'a, Message>>,
    spacing: f32,
    line_spacing: f32,
) -> Element<'a, Message> {
    if direction.is_rtl() {
        children.reverse();
    }
    Row::with_children(children)
        .spacing(spacing)
        .width(Length::Fill)
        .wrap()
        .vertical_spacing(line_spacing)
        .align_x(direction.start())
        .into()
}

/// A card of `max_width` centred by its parent: a `DIALOG_MARGIN` margin keeps it off the
/// viewport edges, and the card pads its content by `DIALOG_PADDING`.
#[must_use]
pub(crate) fn card_frame<'a>(
    ctx: ViewContext,
    content: Element<'a, Message>,
    max_width: f32,
) -> Element<'a, Message> {
    let palette = ctx.palette;
    let card = container(content)
        .padding(DIALOG_PADDING)
        .width(Length::Fill)
        .style(move |_: &Theme| theme::card(palette));
    container(card)
        .padding(DIALOG_MARGIN)
        .width(Length::Fill)
        .max_width(max_width + 2.0 * DIALOG_MARGIN)
        .into()
}

/// Marks `content` as the open surface: Tab and Shift+Tab are scoped to the container with
/// this identifier while a menu, dialog or drawer is open.
#[must_use]
pub(crate) fn open_surface<'a>(content: Element<'a, Message>) -> Element<'a, Message> {
    container(content).id(ids::OPEN_SURFACE).into()
}

#[cfg(test)]
pub(crate) mod test_support {
    //! Contexts for the pane smoke tests.

    use crate::app::context::{Platform, ViewContext};
    use crate::app::i18n::Locale;
    use crate::app::layout::LayoutMode;
    use crate::app::theme::PALETTE_DARK;

    /// Context for a platform, layout mode and locale; the direction follows the locale.
    pub(crate) fn context(platform: Platform, mode: LayoutMode, locale: Locale) -> ViewContext {
        ViewContext {
            palette: PALETTE_DARK,
            locale,
            direction: locale.direction(),
            mode,
            platform,
        }
    }

    /// Every combination of platform, mode and a left-to-right and a right-to-left locale.
    pub(crate) fn contexts() -> Vec<ViewContext> {
        let mut all = Vec::new();
        for platform in [Platform::Desktop, Platform::Browser] {
            for mode in [LayoutMode::Wide, LayoutMode::Narrow] {
                for locale in [Locale::EnUs, Locale::Ar, Locale::KoKr] {
                    all.push(context(platform, mode, locale));
                }
            }
        }
        all
    }
}
