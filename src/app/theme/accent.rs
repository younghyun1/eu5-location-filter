//! Desktop accent presets.

use iced::Color;

use super::palette::Mode;
use crate::app::i18n::Text;

/// Desktop accent presets; each replaces only `accent`, `accent_strong` and `accent_soft`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub(crate) enum AccentPreset {
    /// Site default.
    #[default]
    Amber,
    /// Teal.
    Lagoon,
    /// Blue.
    Cobalt,
    /// Violet.
    Orchid,
    /// Neutral.
    Graphite,
}

impl AccentPreset {
    /// Every preset in menu order.
    pub(crate) const ALL: [Self; 5] = [
        Self::Amber,
        Self::Lagoon,
        Self::Cobalt,
        Self::Orchid,
        Self::Graphite,
    ];

    /// `accent`, `accent_strong` and `accent_soft` for a mode.
    #[must_use]
    pub(crate) const fn tokens(self, mode: Mode) -> [Color; 3] {
        match (self, mode) {
            (Self::Amber, Mode::Light) => [
                Color::from_rgb8(180, 83, 9),
                Color::from_rgb8(146, 64, 14),
                Color::from_rgb8(246, 229, 196),
            ],
            (Self::Amber, Mode::Dark) => [
                Color::from_rgb8(251, 191, 36),
                Color::from_rgb8(252, 211, 77),
                Color::from_rgb8(66, 32, 6),
            ],
            (Self::Lagoon, Mode::Light) => [
                Color::from_rgb8(15, 118, 110),
                Color::from_rgb8(17, 94, 89),
                Color::from_rgb8(224, 240, 235),
            ],
            (Self::Lagoon, Mode::Dark) => [
                Color::from_rgb8(45, 212, 191),
                Color::from_rgb8(94, 234, 212),
                Color::from_rgb8(11, 46, 42),
            ],
            (Self::Cobalt, Mode::Light) => [
                Color::from_rgb8(29, 78, 216),
                Color::from_rgb8(30, 58, 138),
                Color::from_rgb8(221, 230, 248),
            ],
            (Self::Cobalt, Mode::Dark) => [
                Color::from_rgb8(96, 165, 250),
                Color::from_rgb8(147, 197, 253),
                Color::from_rgb8(15, 31, 61),
            ],
            (Self::Orchid, Mode::Light) => [
                Color::from_rgb8(126, 34, 206),
                Color::from_rgb8(88, 28, 135),
                Color::from_rgb8(236, 223, 247),
            ],
            (Self::Orchid, Mode::Dark) => [
                Color::from_rgb8(192, 132, 252),
                Color::from_rgb8(216, 180, 254),
                Color::from_rgb8(42, 19, 64),
            ],
            (Self::Graphite, Mode::Light) => [
                Color::from_rgb8(74, 68, 60),
                Color::from_rgb8(33, 29, 24),
                Color::from_rgb8(230, 223, 211),
            ],
            (Self::Graphite, Mode::Dark) => [
                Color::from_rgb8(214, 208, 198),
                Color::from_rgb8(240, 236, 229),
                Color::from_rgb8(42, 38, 34),
            ],
        }
    }

    /// Catalog label for the Appearance menu.
    #[must_use]
    pub(crate) const fn title(self) -> Text {
        match self {
            Self::Amber => Text::AppearanceAccentAmber,
            Self::Lagoon => Text::AppearanceAccentLagoon,
            Self::Cobalt => Text::AppearanceAccentCobalt,
            Self::Orchid => Text::AppearanceAccentOrchid,
            Self::Graphite => Text::AppearanceAccentGraphite,
        }
    }
}
