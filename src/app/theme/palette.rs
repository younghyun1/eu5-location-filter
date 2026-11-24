//! The 12 site tokens, the built-in palettes, and token-wise mixing.

use iced::Color;

use super::accent::AccentPreset;
use crate::app::i18n::Text;

/// Light or dark site mode.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub(crate) enum Mode {
    /// Light paper.
    Light,
    /// Dark paper; the start mode on both targets.
    #[default]
    Dark,
}

impl Mode {
    /// Parses the host protocol value `"light"` or `"dark"`.
    #[must_use]
    pub(crate) fn from_host(value: &str) -> Option<Self> {
        match value {
            "light" => Some(Self::Light),
            "dark" => Some(Self::Dark),
            _ => None,
        }
    }

    /// Catalog label for the Appearance menu.
    #[must_use]
    pub(crate) const fn title(self) -> Text {
        match self {
            Self::Light => Text::AppearanceModeLight,
            Self::Dark => Text::AppearanceModeDark,
        }
    }
}

/// The host palette: the site's 12 colour tokens and the mode. Field order is the protocol
/// token order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Palette {
    /// Light or dark.
    pub(crate) mode: Mode,
    /// Window ground.
    pub(crate) paper: Color,
    /// Pane and card ground.
    pub(crate) surface: Color,
    /// Primary text.
    pub(crate) ink: Color,
    /// Secondary text.
    pub(crate) ink_muted: Color,
    /// Tertiary text and resting scrollbar thumbs.
    pub(crate) ink_faint: Color,
    /// Hairline borders.
    pub(crate) line: Color,
    /// Stronger borders.
    pub(crate) line_strong: Color,
    /// Accent, focus and selection bar.
    pub(crate) accent: Color,
    /// Strong accent and river cells.
    pub(crate) accent_strong: Color,
    /// Soft accent ground for selection and hover.
    pub(crate) accent_soft: Color,
    /// Positive colour and food-producing goods.
    pub(crate) ok: Color,
    /// Errors.
    pub(crate) danger: Color,
}

/// Light site palette with the amber accent, copied from the site's `index.css`.
pub(crate) const PALETTE_LIGHT: Palette = Palette {
    mode: Mode::Light,
    paper: Color::from_rgb8(246, 241, 232),
    surface: Color::from_rgb8(253, 250, 244),
    ink: Color::from_rgb8(33, 29, 24),
    ink_muted: Color::from_rgb8(110, 101, 90),
    ink_faint: Color::from_rgb8(154, 144, 132),
    line: Color::from_rgb8(227, 218, 203),
    line_strong: Color::from_rgb8(201, 191, 174),
    accent: Color::from_rgb8(180, 83, 9),
    accent_strong: Color::from_rgb8(146, 64, 14),
    accent_soft: Color::from_rgb8(246, 229, 196),
    ok: Color::from_rgb8(4, 120, 87),
    danger: Color::from_rgb8(190, 18, 60),
};

/// Dark site palette with the amber accent, copied from the site's `index.css`.
pub(crate) const PALETTE_DARK: Palette = Palette {
    mode: Mode::Dark,
    paper: Color::from_rgb8(0, 0, 0),
    surface: Color::from_rgb8(20, 18, 16),
    ink: Color::from_rgb8(240, 236, 229),
    ink_muted: Color::from_rgb8(163, 154, 143),
    ink_faint: Color::from_rgb8(111, 103, 93),
    line: Color::from_rgb8(38, 34, 32),
    line_strong: Color::from_rgb8(61, 55, 49),
    accent: Color::from_rgb8(251, 191, 36),
    accent_strong: Color::from_rgb8(252, 211, 77),
    accent_soft: Color::from_rgb8(66, 32, 6),
    ok: Color::from_rgb8(52, 211, 153),
    danger: Color::from_rgb8(251, 113, 133),
};

/// Per-channel linear interpolation of the sRGB components and alpha, `t` clamped to `0..=1`.
/// iced's own `palette::mix` works in linear light and would not match the site's sRGB mixes.
#[must_use]
pub(crate) fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = if t.is_nan() { 0.0 } else { t.clamp(0.0, 1.0) };
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}

impl Palette {
    /// Number of colour tokens in the host protocol.
    pub(crate) const TOKEN_COUNT: usize = 12;

    /// Built-in palette for a mode, amber accent.
    #[must_use]
    pub(crate) const fn builtin(mode: Mode) -> Self {
        match mode {
            Mode::Light => PALETTE_LIGHT,
            Mode::Dark => PALETTE_DARK,
        }
    }

    /// Validates host input: a known mode and exactly 12 values, each at most `0xFFFFFF`, in
    /// protocol order. Returns `None` for anything else.
    #[must_use]
    pub(crate) fn from_host(mode: &str, colours: &[u32]) -> Option<Self> {
        let mode = Mode::from_host(mode)?;
        let values: &[u32; Self::TOKEN_COUNT] = colours.try_into().ok()?;
        let mut tokens = [Color::BLACK; Self::TOKEN_COUNT];
        for (token, value) in tokens.iter_mut().zip(values) {
            let [_, red, green, blue] = value.to_be_bytes();
            if *value > 0x00FF_FFFF {
                return None;
            }
            *token = Color::from_rgb8(red, green, blue);
        }
        Some(Self::from_tokens(mode, tokens))
    }

    /// Builds a palette from tokens in protocol order.
    #[must_use]
    pub(crate) const fn from_tokens(mode: Mode, tokens: [Color; Self::TOKEN_COUNT]) -> Self {
        let [
            paper,
            surface,
            ink,
            ink_muted,
            ink_faint,
            line,
            line_strong,
            accent,
            accent_strong,
            accent_soft,
            ok,
            danger,
        ] = tokens;
        Self {
            mode,
            paper,
            surface,
            ink,
            ink_muted,
            ink_faint,
            line,
            line_strong,
            accent,
            accent_strong,
            accent_soft,
            ok,
            danger,
        }
    }

    /// Tokens in protocol order.
    #[must_use]
    pub(crate) const fn tokens(&self) -> [Color; Self::TOKEN_COUNT] {
        [
            self.paper,
            self.surface,
            self.ink,
            self.ink_muted,
            self.ink_faint,
            self.line,
            self.line_strong,
            self.accent,
            self.accent_strong,
            self.accent_soft,
            self.ok,
            self.danger,
        ]
    }

    /// Token-wise `mix` toward `other`; the mode switches at the midpoint.
    #[must_use]
    pub(crate) fn mixed(&self, other: &Self, t: f32) -> Self {
        let mut tokens = self.tokens();
        for (token, target) in tokens.iter_mut().zip(other.tokens()) {
            *token = mix(*token, target, t);
        }
        let mode = if t < 0.5 { self.mode } else { other.mode };
        Self::from_tokens(mode, tokens)
    }

    /// Replaces the three accent tokens with a preset for this palette's mode.
    #[must_use]
    pub(crate) const fn with_accent(self, preset: AccentPreset) -> Self {
        let [accent, accent_strong, accent_soft] = preset.tokens(self.mode);
        Self {
            accent,
            accent_strong,
            accent_soft,
            ..self
        }
    }

    /// iced theme whose base colours match this palette, for stock widget defaults.
    #[must_use]
    pub(crate) fn iced_theme(&self) -> iced::Theme {
        iced::Theme::custom(
            "EU5 Location DB",
            iced::theme::Palette {
                background: self.paper,
                text: self.ink,
                primary: self.accent,
                success: self.ok,
                warning: self.accent_strong,
                danger: self.danger,
            },
        )
    }
}
