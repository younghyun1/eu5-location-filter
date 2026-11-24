//! Host palette parsing, token order, mixing and accent presets.

use iced::Color;

use super::common::{close, colors_close};
use crate::app::theme::{AccentPreset, Mode, PALETTE_DARK, PALETTE_LIGHT, Palette, mix};

/// Twelve distinct opaque values in protocol order.
const HOST_VALUES: [u32; 12] = [
    0x00F6_F1E8,
    0x00FD_FAF4,
    0x0021_1D18,
    0x006E_655A,
    0x009A_9084,
    0x00E3_DACB,
    0x00C9_BFAE,
    0x00B4_5309,
    0x0092_400E,
    0x00F6_E5C4,
    0x0004_7857,
    0x00BE_123C,
];

#[test]
fn valid_host_palette_maps_values_in_protocol_order() {
    let expected = Palette {
        mode: Mode::Light,
        paper: Color::from_rgb8(0xF6, 0xF1, 0xE8),
        surface: Color::from_rgb8(0xFD, 0xFA, 0xF4),
        ink: Color::from_rgb8(0x21, 0x1D, 0x18),
        ink_muted: Color::from_rgb8(0x6E, 0x65, 0x5A),
        ink_faint: Color::from_rgb8(0x9A, 0x90, 0x84),
        line: Color::from_rgb8(0xE3, 0xDA, 0xCB),
        line_strong: Color::from_rgb8(0xC9, 0xBF, 0xAE),
        accent: Color::from_rgb8(0xB4, 0x53, 0x09),
        accent_strong: Color::from_rgb8(0x92, 0x40, 0x0E),
        accent_soft: Color::from_rgb8(0xF6, 0xE5, 0xC4),
        ok: Color::from_rgb8(0x04, 0x78, 0x57),
        danger: Color::from_rgb8(0xBE, 0x12, 0x3C),
    };
    assert_eq!(Palette::from_host("light", &HOST_VALUES), Some(expected));
    assert_eq!(Palette::from_host("light", &HOST_VALUES), Some(PALETTE_LIGHT));
}

#[test]
fn dark_mode_is_parsed_and_kept() {
    let parsed = Palette::from_host("dark", &HOST_VALUES);
    assert_eq!(parsed.map(|palette| palette.mode), Some(Mode::Dark));
}

#[test]
fn wrong_token_count_is_rejected() {
    assert_eq!(Palette::from_host("dark", &[]), None);
    assert_eq!(Palette::from_host("dark", &HOST_VALUES[..11]), None);
    let mut long = HOST_VALUES.to_vec();
    long.push(0);
    assert_eq!(Palette::from_host("dark", &long), None);
}

#[test]
fn unknown_or_miscased_mode_is_rejected() {
    for mode in ["", "auto", "Dark", "LIGHT", "light ", "dim"] {
        assert_eq!(Palette::from_host(mode, &HOST_VALUES), None, "mode {mode:?}");
    }
    assert_eq!(Mode::from_host("light"), Some(Mode::Light));
    assert_eq!(Mode::from_host("dark"), Some(Mode::Dark));
    assert_eq!(Mode::from_host("system"), None);
}

#[test]
fn values_above_24_bits_are_rejected() {
    let mut values = HOST_VALUES;
    values[5] = 0x0100_0000;
    assert_eq!(Palette::from_host("dark", &values), None);
    values[5] = 0xFF00_0000;
    assert_eq!(Palette::from_host("dark", &values), None);
    values[5] = 0x00FF_FFFF;
    assert!(Palette::from_host("dark", &values).is_some());
}

#[test]
fn start_mode_is_dark() {
    assert_eq!(Mode::default(), Mode::Dark);
    assert_eq!(Palette::builtin(Mode::Dark), PALETTE_DARK);
    assert_eq!(Palette::builtin(Mode::Light), PALETTE_LIGHT);
}

#[test]
fn tokens_round_trip_in_protocol_order() {
    for base in [PALETTE_LIGHT, PALETTE_DARK] {
        let tokens = base.tokens();
        assert_eq!(tokens.len(), Palette::TOKEN_COUNT);
        assert_eq!(tokens.first(), Some(&base.paper));
        assert_eq!(tokens.last(), Some(&base.danger));
        assert_eq!(Palette::from_tokens(base.mode, tokens), base);
    }
}

#[test]
fn mix_returns_its_endpoints() {
    let a = Color::from_rgb8(10, 20, 30);
    let b = Color::from_rgb8(200, 150, 100);
    assert_eq!(mix(a, b, 0.0), a);
    assert_eq!(mix(a, b, 1.0), b);
}

#[test]
fn mix_midpoint_averages_each_channel_and_alpha() {
    let a = Color::from_rgba(0.0, 0.2, 1.0, 0.0);
    let b = Color::from_rgba(1.0, 0.6, 1.0, 1.0);
    let middle = mix(a, b, 0.5);
    assert!(colors_close(middle, Color::from_rgba(0.5, 0.4, 1.0, 0.5)));
}

#[test]
fn mix_works_in_srgb_not_linear_light() {
    let middle = mix(Color::BLACK, Color::WHITE, 0.5);
    assert!(close(middle.r, 0.5) && close(middle.g, 0.5) && close(middle.b, 0.5));
}

#[test]
fn mix_clamps_and_survives_nan() {
    let a = Color::from_rgb8(10, 20, 30);
    let b = Color::from_rgb8(200, 150, 100);
    assert_eq!(mix(a, b, -3.0), a);
    assert_eq!(mix(a, b, 7.0), b);
    assert_eq!(mix(a, b, f32::NAN), a);
}

#[test]
fn mixed_palette_hits_both_ends_and_switches_mode_at_the_midpoint() {
    assert_eq!(PALETTE_DARK.mixed(&PALETTE_LIGHT, 0.0), PALETTE_DARK);
    assert_eq!(PALETTE_DARK.mixed(&PALETTE_LIGHT, 1.0), PALETTE_LIGHT);
    assert_eq!(PALETTE_DARK.mixed(&PALETTE_LIGHT, 0.49).mode, Mode::Dark);
    assert_eq!(PALETTE_DARK.mixed(&PALETTE_LIGHT, 0.5).mode, Mode::Light);
    let middle = PALETTE_DARK.mixed(&PALETTE_LIGHT, 0.5);
    assert!(colors_close(middle.paper, mix(PALETTE_DARK.paper, PALETTE_LIGHT.paper, 0.5)));
    assert!(colors_close(middle.danger, mix(PALETTE_DARK.danger, PALETTE_LIGHT.danger, 0.5)));
}

#[test]
fn amber_preset_is_the_built_in_accent() {
    for base in [PALETTE_LIGHT, PALETTE_DARK] {
        assert_eq!(base.with_accent(AccentPreset::Amber), base);
    }
    assert_eq!(AccentPreset::default(), AccentPreset::Amber);
}

#[test]
fn preset_replaces_only_the_three_accent_tokens() {
    for base in [PALETTE_LIGHT, PALETTE_DARK] {
        for preset in AccentPreset::ALL {
            let tinted = base.with_accent(preset);
            let [accent, strong, soft] = preset.tokens(base.mode);
            assert_eq!((tinted.accent, tinted.accent_strong, tinted.accent_soft), (accent, strong, soft));
            assert_eq!(tinted.mode, base.mode);
            assert_eq!(tinted.paper, base.paper);
            assert_eq!(tinted.surface, base.surface);
            assert_eq!(tinted.ink, base.ink);
            assert_eq!(tinted.line, base.line);
            assert_eq!(tinted.ok, base.ok);
            assert_eq!(tinted.danger, base.danger);
        }
    }
}

#[test]
fn presets_are_distinct_in_every_mode() {
    for mode in [Mode::Light, Mode::Dark] {
        for (index, first) in AccentPreset::ALL.iter().enumerate() {
            for second in AccentPreset::ALL.iter().skip(index + 1) {
                assert_ne!(first.tokens(mode), second.tokens(mode), "{first:?} vs {second:?}");
            }
        }
    }
}

#[test]
fn graphite_and_cobalt_match_the_layout_table() {
    assert_eq!(
        AccentPreset::Graphite.tokens(Mode::Light),
        [
            Color::from_rgb8(0x4A, 0x44, 0x3C),
            Color::from_rgb8(0x21, 0x1D, 0x18),
            Color::from_rgb8(0xE6, 0xDF, 0xD3),
        ]
    );
    assert_eq!(
        AccentPreset::Cobalt.tokens(Mode::Dark),
        [
            Color::from_rgb8(0x60, 0xA5, 0xFA),
            Color::from_rgb8(0x93, 0xC5, 0xFD),
            Color::from_rgb8(0x0F, 0x1F, 0x3D),
        ]
    );
}

#[test]
fn iced_theme_carries_the_palette_base_colours() {
    let theme = PALETTE_DARK.iced_theme();
    let base = theme.palette();
    assert_eq!(base.background, PALETTE_DARK.paper);
    assert_eq!(base.text, PALETTE_DARK.ink);
    assert_eq!(base.primary, PALETTE_DARK.accent);
    assert_eq!(base.success, PALETTE_DARK.ok);
    assert_eq!(base.danger, PALETTE_DARK.danger);
}
