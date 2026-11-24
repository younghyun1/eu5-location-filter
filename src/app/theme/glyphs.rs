//! Icon glyphs from iced's built-in icon font and the two Plex Sans symbols, chosen by direction.

use iced::Font;
use iced::advanced::text::Renderer as TextRenderer;

use super::fonts::SANS;
use crate::app::layout::Direction;

/// Every glyph the interface draws. No directional glyph is stored in a catalog.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Icon {
    /// Checked box.
    Check,
    /// Ascending sort arrow.
    SortAscending,
    /// Descending sort arrow, also the expanded facet group chevron.
    SortDescending,
    /// Folded facet group chevron, pointing toward the inline end.
    Folded,
    /// Chevron pointing toward the inline start (collapse the rail, hide toward start).
    TowardStart,
    /// Chevron pointing toward the inline end.
    TowardEnd,
    /// Clear and close: U+00D7 multiplication sign from Plex Sans.
    Close,
    /// Breadcrumb separator: U+203A, or U+2039 right to left, from Plex Sans.
    Breadcrumb,
}

impl Icon {
    /// Character to draw for this direction.
    #[must_use]
    pub(crate) const fn glyph(self, direction: Direction) -> char {
        let rtl = direction.is_rtl();
        match self {
            Self::Check => <iced::Renderer as TextRenderer>::CHECKMARK_ICON,
            Self::SortAscending => <iced::Renderer as TextRenderer>::SCROLL_UP_ICON,
            Self::SortDescending => <iced::Renderer as TextRenderer>::SCROLL_DOWN_ICON,
            Self::Folded | Self::TowardEnd => {
                if rtl {
                    <iced::Renderer as TextRenderer>::SCROLL_LEFT_ICON
                } else {
                    <iced::Renderer as TextRenderer>::SCROLL_RIGHT_ICON
                }
            }
            Self::TowardStart => {
                if rtl {
                    <iced::Renderer as TextRenderer>::SCROLL_RIGHT_ICON
                } else {
                    <iced::Renderer as TextRenderer>::SCROLL_LEFT_ICON
                }
            }
            Self::Close => '\u{00D7}',
            Self::Breadcrumb => {
                if rtl {
                    '\u{2039}'
                } else {
                    '\u{203A}'
                }
            }
        }
    }

    /// Font holding the glyph.
    #[must_use]
    pub(crate) const fn font(self) -> Font {
        match self {
            Self::Close | Self::Breadcrumb => SANS,
            Self::Check
            | Self::SortAscending
            | Self::SortDescending
            | Self::Folded
            | Self::TowardStart
            | Self::TowardEnd => <iced::Renderer as TextRenderer>::ICON_FONT,
        }
    }
}
