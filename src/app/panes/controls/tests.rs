//! Tests of the shared controls: fixed sizes and construction in every layout mode and direction.

use iced::{Element, Length, Size};

use super::*;
use crate::app::context::{Platform, ViewContext};
use crate::app::i18n::{Locale, Text};
use crate::app::ids;
use crate::app::layout::{
    BUTTON_HEIGHT, BUTTON_WIDTH, Direction, ICON_BUTTON_SIZE, LayoutMode, TOUCH_TARGET,
};
use crate::app::message::{FilterMessage, Message};
use crate::app::theme::{ButtonKind, Icon, PALETTE_DARK};

fn context(direction: Direction, mode: LayoutMode) -> ViewContext {
    ViewContext {
        palette: PALETTE_DARK,
        locale: Locale::EnUs,
        direction,
        mode,
        platform: Platform::Desktop,
    }
}

fn every_context() -> [ViewContext; 4] {
    [
        context(Direction::Ltr, LayoutMode::Wide),
        context(Direction::Ltr, LayoutMode::Narrow),
        context(Direction::Rtl, LayoutMode::Wide),
        context(Direction::Rtl, LayoutMode::Narrow),
    ]
}

#[test]
fn text_button_size_ignores_the_label() {
    for label in ["OK", "A label far longer than the button can show at its width"] {
        let wide = context(Direction::Ltr, LayoutMode::Wide);
        let element: Element<'_, Message> =
            text_button(wide, label, ButtonKind::Secondary, BUTTON_WIDTH, None).into();
        assert_eq!(
            element.as_widget().size(),
            Size::new(Length::Fixed(BUTTON_WIDTH), Length::Fixed(BUTTON_HEIGHT))
        );
    }
}

#[test]
fn narrow_buttons_use_the_touch_target() {
    let narrow = context(Direction::Ltr, LayoutMode::Narrow);
    let text: Element<'_, Message> =
        text_button(narrow, "Columns", ButtonKind::Secondary, BUTTON_WIDTH, None).into();
    assert_eq!(
        text.as_widget().size(),
        Size::new(Length::Fixed(BUTTON_WIDTH), Length::Fixed(TOUCH_TARGET))
    );
    let icon: Element<'_, Message> = icon_button(narrow, Icon::Close, "Close", None).into();
    assert_eq!(
        icon.as_widget().size(),
        Size::new(Length::Fixed(TOUCH_TARGET), Length::Fixed(TOUCH_TARGET))
    );
}

#[test]
fn wide_icon_button_is_square() {
    let wide = context(Direction::Rtl, LayoutMode::Wide);
    let icon: Element<'_, Message> = icon_button(wide, Icon::TowardStart, "Hide", None).into();
    assert_eq!(
        icon.as_widget().size(),
        Size::new(Length::Fixed(ICON_BUTTON_SIZE), Length::Fixed(ICON_BUTTON_SIZE))
    );
}

#[test]
fn button_frame_keeps_the_requested_width() {
    let ctx = context(Direction::Ltr, LayoutMode::Wide);
    let element: Element<'_, Message> = button_frame(
        ctx,
        button_label(ctx, "Filters"),
        "Filters",
        ButtonKind::Secondary,
        Length::Fill,
        None,
    )
    .into();
    assert_eq!(element.as_widget().size().width, Length::Fill);
}

#[test]
fn every_control_builds_in_every_context() {
    for ctx in every_context() {
        let _: Element<'_, Message> = badge(ctx, 12);
        let _: Element<'_, Message> = caption(ctx, "Caption");
        let _: Element<'_, Message> = small_label(ctx, "Label");
        let _: Element<'_, Message> = field_error(ctx, "Error");
        let _: Element<'_, Message> = with_tooltip(ctx, caption(ctx, "x"), "tip");
        let _: Element<'_, Message> = with_tooltip_owned(ctx, caption(ctx, "x"), "tip".to_owned());
        let _: Element<'_, Message> = text_field(
            ctx,
            ids::RGB_FIELD,
            "placeholder",
            "value",
            true,
            |text| Message::Filter(FilterMessage::Rgb(text)),
        );
        let _: Element<'_, Message> = toggle_row(ctx, "Toggle", true, |on| {
            Message::Filter(FilterMessage::FoodOnly(on))
        });
        let _: Element<'_, Message> =
            menu_frame(ctx, "Menu", "Close", caption(ctx, "Content"));
    }
}

#[test]
fn counted_text_follows_the_plural_form() {
    let ctx = context(Direction::Ltr, LayoutMode::Wide);
    assert_eq!(counted(ctx, Text::RailActiveCount, 1), "1 filter active");
    assert_eq!(counted(ctx, Text::RailActiveCount, 3), "3 filters active");
    assert_eq!(counted(ctx, Text::HeaderResultCount, 1234), "1,234 locations");
}

#[test]
fn script_pack_locales_use_their_own_face_for_mono_text() {
    let mut ctx = context(Direction::Ltr, LayoutMode::Wide);
    assert_eq!(mono_font(ctx, true), crate::app::theme::MONO_MEDIUM);
    assert_eq!(mono_font(ctx, false), crate::app::theme::MONO);
    ctx.locale = Locale::JaJp;
    assert_ne!(mono_font(ctx, true), crate::app::theme::MONO_MEDIUM);
    assert_eq!(
        mono_font(ctx, false),
        sans_font(ctx, iced::font::Weight::Normal)
    );
}
