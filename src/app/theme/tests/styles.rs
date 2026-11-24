//! Status handling of the widget and container style functions.

use iced::widget::{button, checkbox, scrollable, text_input};
use iced::{Background, Color};

use super::common::close;
use crate::app::layout::FOCUS_RING_WIDTH;
use crate::app::theme::{
    self, ButtonKind, PALETTE_DARK, PALETTE_LIGHT, Palette, ThumbState, Veil,
};

const KINDS: [ButtonKind; 3] = [ButtonKind::Primary, ButtonKind::Secondary, ButtonKind::Ghost];
const BUTTON_STATUSES: [button::Status; 4] = [
    button::Status::Active,
    button::Status::Hovered,
    button::Status::Pressed,
    button::Status::Disabled,
];

fn ground(style: button::Style) -> Option<Color> {
    match style.background {
        Some(Background::Color(color)) => Some(color),
        _ => None,
    }
}

#[test]
fn primary_button_uses_accent_and_strengthens_on_hover_and_press() {
    for palette in [PALETTE_LIGHT, PALETTE_DARK] {
        let rest = theme::button(palette, ButtonKind::Primary, button::Status::Active);
        assert_eq!(ground(rest), Some(palette.accent));
        assert_eq!(rest.text_color, palette.on_accent());
        assert!(close(rest.border.width, 0.0));
        for status in [button::Status::Hovered, button::Status::Pressed] {
            let style = theme::button(palette, ButtonKind::Primary, status);
            assert_eq!(ground(style), Some(palette.accent_strong));
            assert_eq!(style.text_color, palette.on_accent());
        }
    }
}

#[test]
fn disabled_primary_button_looks_like_a_disabled_button() {
    let palette = PALETTE_LIGHT;
    let primary = theme::button(palette, ButtonKind::Primary, button::Status::Disabled);
    let secondary = theme::button(palette, ButtonKind::Secondary, button::Status::Disabled);
    assert_eq!(primary, secondary);
    assert_eq!(primary.text_color, palette.ink_faint);
    assert_eq!(primary.border.color, palette.line);
}

#[test]
fn secondary_button_follows_the_control_table() {
    let palette = PALETTE_DARK;
    let rest = theme::button(palette, ButtonKind::Secondary, button::Status::Active);
    assert_eq!(ground(rest), Some(palette.surface));
    assert_eq!(rest.border.color, palette.line_strong);
    assert_eq!(rest.text_color, palette.ink);
    let hover = theme::button(palette, ButtonKind::Secondary, button::Status::Hovered);
    assert_eq!(ground(hover), Some(palette.surface_2()));
    let pressed = theme::button(palette, ButtonKind::Secondary, button::Status::Pressed);
    assert_eq!(ground(pressed), Some(palette.pressed()));
}

#[test]
fn ghost_button_has_no_ground_until_hovered() {
    let palette = PALETTE_LIGHT;
    let rest = theme::button(palette, ButtonKind::Ghost, button::Status::Active);
    assert_eq!(rest.background, None);
    assert_eq!(rest.text_color, palette.ink_muted);
    let hover = theme::button(palette, ButtonKind::Ghost, button::Status::Hovered);
    assert_eq!(ground(hover), Some(palette.surface_2()));
    assert_eq!(hover.text_color, palette.ink);
    let pressed = theme::button(palette, ButtonKind::Ghost, button::Status::Pressed);
    assert_eq!(ground(pressed), Some(palette.pressed()));
    let disabled = theme::button(palette, ButtonKind::Ghost, button::Status::Disabled);
    assert_eq!(disabled.background, None);
    assert_eq!(disabled.text_color, palette.ink_faint);
}

#[test]
fn buttons_never_cast_shadows() {
    for kind in KINDS {
        for status in BUTTON_STATUSES {
            let style = theme::button(PALETTE_LIGHT, kind, status);
            assert_eq!(style.shadow, iced::Shadow::default());
        }
    }
}

fn input(palette: Palette, invalid: bool, status: text_input::Status) -> text_input::Style {
    theme::text_input(palette, invalid, status)
}

#[test]
fn text_field_borders_follow_the_control_table() {
    let palette = PALETTE_LIGHT;
    let rest = input(palette, false, text_input::Status::Active);
    assert_eq!(rest.border.color, palette.line_strong);
    assert_eq!(rest.background, Background::Color(palette.surface));
    assert_eq!(rest.placeholder, palette.ink_faint);
    assert_eq!(rest.value, palette.ink);
    let hover = input(palette, false, text_input::Status::Hovered);
    assert_eq!(hover.border.color, palette.ink_faint);
    for is_hovered in [false, true] {
        let focused = input(palette, false, text_input::Status::Focused { is_hovered });
        assert_eq!(focused.border.color, palette.accent);
        assert!(close(focused.border.width, FOCUS_RING_WIDTH));
    }
    let disabled = input(palette, false, text_input::Status::Disabled);
    assert_eq!(disabled.value, palette.ink_faint);
    assert_eq!(disabled.border.color, palette.line);
}

#[test]
fn invalid_text_field_uses_danger_in_every_enabled_status() {
    let palette = PALETTE_DARK;
    for status in [
        text_input::Status::Active,
        text_input::Status::Hovered,
        text_input::Status::Focused { is_hovered: false },
        text_input::Status::Focused { is_hovered: true },
    ] {
        assert_eq!(input(palette, true, status).border.color, palette.danger);
    }
    let focused = input(palette, true, text_input::Status::Focused { is_hovered: false });
    assert!(close(focused.border.width, FOCUS_RING_WIDTH));
    let rest = input(palette, true, text_input::Status::Active);
    assert!(rest.border.width < FOCUS_RING_WIDTH);
}

#[test]
fn checkbox_states_follow_the_control_table() {
    let palette = PALETTE_LIGHT;
    let style = |status| theme::checkbox(palette, status);
    let rest = style(checkbox::Status::Active { is_checked: false });
    assert_eq!(rest.border.color, palette.line_strong);
    let hover = style(checkbox::Status::Hovered { is_checked: false });
    assert_eq!(hover.border.color, palette.ink_faint);
    let disabled = style(checkbox::Status::Disabled { is_checked: false });
    assert_eq!(disabled.border.color, palette.line);
    assert_eq!(disabled.text_color, Some(palette.ink_faint));

    let checked = style(checkbox::Status::Active { is_checked: true });
    assert_eq!(checked.background, Background::Color(palette.accent));
    assert_eq!(checked.icon_color, palette.on_accent());
    let checked_hover = style(checkbox::Status::Hovered { is_checked: true });
    assert_eq!(checked_hover.background, Background::Color(palette.accent_strong));
    let checked_disabled = style(checkbox::Status::Disabled { is_checked: true });
    assert_eq!(checked_disabled.background, Background::Color(palette.line_strong));
}

#[test]
fn scrollbar_thumbs_follow_pointer_state() {
    let palette = PALETTE_DARK;
    let thumb = |style: scrollable::Style| (style.vertical_rail.scroller.background, style.horizontal_rail.scroller.background);
    let color = |state| Background::Color(palette.scrollbar_thumb(state));

    let active = scrollable::Status::Active {
        is_horizontal_scrollbar_disabled: false,
        is_vertical_scrollbar_disabled: false,
    };
    assert_eq!(thumb(theme::scrollable(palette, active)), (color(ThumbState::Idle), color(ThumbState::Idle)));

    let hovered = scrollable::Status::Hovered {
        is_horizontal_scrollbar_hovered: false,
        is_vertical_scrollbar_hovered: true,
        is_horizontal_scrollbar_disabled: false,
        is_vertical_scrollbar_disabled: false,
    };
    assert_eq!(thumb(theme::scrollable(palette, hovered)), (color(ThumbState::Hovered), color(ThumbState::Idle)));

    let dragged = scrollable::Status::Dragged {
        is_horizontal_scrollbar_dragged: true,
        is_vertical_scrollbar_dragged: false,
        is_horizontal_scrollbar_disabled: false,
        is_vertical_scrollbar_disabled: false,
    };
    assert_eq!(thumb(theme::scrollable(palette, dragged)), (color(ThumbState::Idle), color(ThumbState::Dragged)));
}

#[test]
fn rule_progress_and_text_styles_use_their_tokens() {
    let palette = PALETTE_LIGHT;
    assert_eq!(theme::rule(palette, false).color, palette.line);
    assert_eq!(theme::rule(palette, true).color, palette.line_strong);
    let bar = theme::progress_bar(palette);
    assert_eq!(bar.background, Background::Color(palette.line));
    assert_eq!(bar.bar, Background::Color(palette.accent));
    assert_eq!(theme::text_color(palette.danger).color, Some(palette.danger));
}

#[test]
fn only_floating_surfaces_cast_shadows() {
    let palette = PALETTE_DARK;
    for style in [theme::card(palette), theme::menu(palette), theme::tooltip(palette)] {
        assert_eq!(style.shadow, palette.shadow());
    }
    for style in [
        theme::surface(palette),
        theme::header(palette),
        theme::toolbar(palette),
        theme::badge(palette),
        theme::notice(palette),
        theme::detail_block(palette),
        theme::scrim(palette, 1.0),
        theme::veil(palette, Veil::Loading),
    ] {
        assert_eq!(style.shadow, iced::Shadow::default());
    }
}

#[test]
fn surfaces_use_the_documented_grounds() {
    let palette = PALETTE_LIGHT;
    let ground = |style: iced::widget::container::Style| match style.background {
        Some(Background::Color(color)) => Some(color),
        _ => None,
    };
    assert_eq!(ground(theme::surface(palette)), Some(palette.surface));
    assert_eq!(ground(theme::toolbar(palette)), Some(palette.surface_2()));
    assert_eq!(ground(theme::badge(palette)), Some(palette.accent_soft));
    assert_eq!(ground(theme::detail_block(palette)), Some(palette.surface_2()));
    assert_eq!(ground(theme::veil(palette, Veil::Loading)), Some(palette.loading_veil()));
    assert_eq!(ground(theme::veil(palette, Veil::Error)), Some(palette.error_veil()));
    let swatch = theme::swatch(palette, palette.ok);
    assert_eq!(ground(swatch), Some(palette.ok));
    assert_eq!(swatch.border.color, palette.line_strong);
    assert_eq!(theme::card(palette).border.color, palette.line_strong);
}

#[test]
fn scrim_opacity_scales_and_clamps_its_alpha() {
    let palette = PALETTE_DARK;
    let alpha = |opacity: f32| match theme::scrim(palette, opacity).background {
        Some(Background::Color(color)) => color.a,
        _ => -1.0,
    };
    assert!(close(alpha(0.0), 0.0));
    assert!(close(alpha(1.0), palette.scrim().a));
    assert!(close(alpha(2.0), palette.scrim().a));
    assert!(close(alpha(-1.0), 0.0));
}
