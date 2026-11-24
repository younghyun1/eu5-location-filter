//! Desktop Language and Appearance menus; the browser build never shows them.

use iced::alignment::Vertical;
use iced::font::Weight;
use iced::widget::text::Wrapping;
use iced::widget::{Column, button, container, space, text};
use iced::{Background, Border, Element, Font, Length, Padding, Theme};

use crate::GameLanguage;
use crate::app::context::ViewContext;
use crate::app::i18n::{Locale, Text};
use crate::app::layout::anchored_text;
use crate::app::layout::{
    BORDER_WIDTH, BUTTON_HEIGHT, BUTTON_PADDING_INLINE, FOCUS_RING_WIDTH, ICON_BUTTON_SIZE,
    MENU_PADDING, RADIUS, TOUCH_TARGET, inline_row,
};
use crate::app::message::{Message, SettingsMessage};
use crate::app::panes::controls::{menu_frame, with_tooltip};
use crate::app::panes::overlays::kit::{open_surface, prose, wrapped};
use crate::app::state::Appearance;
use crate::app::theme::{self, AccentPreset, ButtonKind, Mode, TEXT_BODY, TEXT_SMALL, sans_for};
use crate::app::widget::focus_ring;

/// Whether the locale shows English game text, so the Language menu explains that location
/// names may stay English. English itself needs no note.
#[must_use]
pub(super) fn lacks_game_text(locale: Locale) -> bool {
    locale != Locale::EnUs && locale.game_language() == GameLanguage::English
}

/// Height of a choice: `BUTTON_HEIGHT`, or `TOUCH_TARGET` in the narrow layout.
fn choice_height(ctx: ViewContext) -> f32 {
    if ctx.is_narrow() {
        TOUCH_TARGET
    } else {
        BUTTON_HEIGHT
    }
}

/// One selectable row or cell: the selected choice is a primary button, the others ghost
/// buttons. The press message is also the focus ring's activation message.
fn choice<'a>(
    ctx: ViewContext,
    label: &'a str,
    font: Font,
    selected: bool,
    on_press: Message,
) -> Element<'a, Message> {
    let palette = ctx.palette;
    let kind = if selected {
        ButtonKind::Primary
    } else {
        ButtonKind::Ghost
    };
    let content = container(anchored_text::<Message>(
        text(label)
            .size(TEXT_BODY)
            .font(font)
            .wrapping(Wrapping::None),
        ctx.direction.start(),
    ))
    .center_y(Length::Fill);
    let body = button(content)
        .width(Length::Fill)
        .height(Length::Fixed(choice_height(ctx)))
        .padding(Padding::ZERO.horizontal(BUTTON_PADDING_INLINE))
        .style(move |_: &Theme, status| theme::button(palette, kind, status))
        .on_press(on_press.clone());
    focus_ring(body)
        .palette(palette)
        .on_activate(on_press)
        .into()
}

/// Round-or-square colour swatch button for one accent preset, with its name as the tooltip.
/// The selected preset has a `FOCUS_RING_WIDTH` `ink` border.
fn accent_swatch<'a>(
    ctx: ViewContext,
    preset: AccentPreset,
    appearance: &Appearance,
) -> Element<'a, Message> {
    let palette = ctx.palette;
    let [accent, _, _] = preset.tokens(appearance.mode);
    let selected = appearance.accent == preset;
    let side = if ctx.is_narrow() {
        TOUCH_TARGET
    } else {
        ICON_BUTTON_SIZE
    };
    let on_press = Message::Settings(SettingsMessage::Accent(preset));
    let body = button(space())
        .width(Length::Fixed(side))
        .height(Length::Fixed(side))
        .style(move |_: &Theme, _status| iced::widget::button::Style {
            background: Some(Background::Color(accent)),
            text_color: palette.on_accent(),
            border: Border {
                color: if selected {
                    palette.ink
                } else {
                    palette.line_strong
                },
                width: if selected {
                    FOCUS_RING_WIDTH
                } else {
                    BORDER_WIDTH
                },
                radius: RADIUS.into(),
            },
            ..iced::widget::button::Style::default()
        })
        .on_press(on_press.clone());
    let control = focus_ring(body)
        .palette(palette)
        .on_activate(on_press)
        .into();
    with_tooltip(ctx, control, ctx.t(preset.title()))
}

/// A labelled group: a small heading above its content.
fn section<'a>(
    ctx: ViewContext,
    label: Text,
    content: Element<'a, Message>,
) -> Element<'a, Message> {
    Column::new()
        .push(prose(ctx, ctx.t(label), TEXT_SMALL, ctx.palette.ink_muted))
        .push(content)
        .spacing(MENU_PADDING / 2.0)
        .width(Length::Fill)
        .into()
}

/// Choices that share a row equally, in inline order.
fn choice_row<'a>(ctx: ViewContext, choices: Vec<Element<'a, Message>>) -> Element<'a, Message> {
    inline_row(ctx.direction, choices)
        .spacing(MENU_PADDING / 2.0)
        .width(Length::Fill)
        .into()
}

/// Every locale by its native name; selecting one publishes `SettingsMessage::Locale`. Shows
/// `language.game_text_note` when the current locale has no game text.
#[must_use]
pub(crate) fn language_menu<'a>(ctx: ViewContext, current: Locale) -> Element<'a, Message> {
    let mut rows = Column::new().width(Length::Fill);
    for locale in Locale::ALL {
        // Each name is set in its own script's face, so that it reads even while another
        // locale's face is the interface font.
        rows = rows.push(choice(
            ctx,
            locale.native_name(),
            sans_for(locale, Weight::Normal),
            locale == current,
            Message::Settings(SettingsMessage::Locale(locale)),
        ));
    }
    if lacks_game_text(current) {
        rows = rows.push(space().height(MENU_PADDING)).push(prose(
            ctx,
            ctx.t(Text::LanguageGameTextNote),
            TEXT_SMALL,
            ctx.palette.ink_muted,
        ));
    }
    open_surface(menu_frame(
        ctx,
        ctx.t(Text::LanguageTitle),
        ctx.t(Text::LanguageClose),
        rows.into(),
    ))
}

/// Mode, accent swatches and animations choices.
#[must_use]
pub(crate) fn appearance_menu<'a>(
    ctx: ViewContext,
    appearance: &'a Appearance,
) -> Element<'a, Message> {
    let font = sans_for(ctx.locale, Weight::Normal);
    let modes = [Mode::Light, Mode::Dark]
        .into_iter()
        .map(|mode| {
            choice(
                ctx,
                ctx.t(mode.title()),
                font,
                appearance.mode == mode,
                Message::Settings(SettingsMessage::Mode(mode)),
            )
        })
        .collect();
    let motions = [
        (true, Text::AppearanceMotionOn),
        (false, Text::AppearanceMotionOff),
    ]
    .into_iter()
    .map(|(on, label)| {
        choice(
            ctx,
            ctx.t(label),
            font,
            appearance.motion_enabled == on,
            Message::Settings(SettingsMessage::Motion(on)),
        )
    })
    .collect();
    let swatches: Vec<Element<'a, Message>> = AccentPreset::ALL
        .into_iter()
        .map(|preset| accent_swatch(ctx, preset, appearance))
        .collect();
    let body = Column::new()
        .push(section(ctx, Text::AppearanceMode, choice_row(ctx, modes)))
        .push(section(
            ctx,
            Text::AppearanceAccent,
            wrapped(ctx.direction, swatches, MENU_PADDING, MENU_PADDING),
        ))
        .push(section(
            ctx,
            Text::AppearanceMotion,
            choice_row(ctx, motions),
        ))
        .spacing(MENU_PADDING * 2.0)
        .width(Length::Fill);
    open_surface(menu_frame(
        ctx,
        ctx.t(Text::AppearanceTitle),
        ctx.t(Text::AppearanceClose),
        body.into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::{appearance_menu, lacks_game_text, language_menu};
    use crate::app::i18n::Locale;
    use crate::app::panes::overlays::kit::test_support::contexts;
    use crate::app::state::Appearance;
    use crate::app::theme::PALETTE_LIGHT;

    #[test]
    fn only_locales_without_game_text_get_the_note() {
        assert!(!lacks_game_text(Locale::EnUs));
        assert!(!lacks_game_text(Locale::KoKr));
        assert!(!lacks_game_text(Locale::FrFr));
        assert!(lacks_game_text(Locale::Ar));
        assert!(lacks_game_text(Locale::HiIn));
    }

    #[test]
    fn the_note_matches_the_game_language_table() {
        for locale in Locale::ALL {
            let expected = locale != Locale::EnUs && locale.game_language().is_embedded();
            assert_eq!(lacks_game_text(locale), expected, "{locale:?}");
        }
    }

    #[test]
    fn the_menus_build_for_every_context() {
        let dark = Appearance::new(crate::app::theme::PALETTE_DARK);
        let light = Appearance::new(PALETTE_LIGHT);
        for ctx in contexts() {
            for current in [Locale::EnUs, Locale::Ar, Locale::KoKr] {
                let _language = language_menu(ctx, current);
            }
            let _dark = appearance_menu(ctx, &dark);
            let _light = appearance_menu(ctx, &light);
        }
    }
}
