//! Font values for the embedded IBM Plex faces and the per-locale script packs.
//!
//! Only `Weight::Normal`, `Medium` and `Semibold` and the normal style and stretch are used: a
//! request that no loaded face matches in style or stretch can leave cosmic-text without any
//! candidate face.

use iced::Font;
use iced::font::{Family, Weight};

use super::constants::{FONT_MONO_FAMILY, FONT_SANS_FAMILY};
use crate::app::i18n::Locale;

/// IBM Plex Sans Regular, the default font of the application.
pub(crate) const SANS: Font = Font {
    family: Family::Name(FONT_SANS_FAMILY),
    ..Font::DEFAULT
};
/// IBM Plex Mono Regular.
pub(crate) const MONO: Font = Font {
    family: Family::Name(FONT_MONO_FAMILY),
    ..Font::DEFAULT
};
/// IBM Plex Mono Medium.
pub(crate) const MONO_MEDIUM: Font = Font {
    family: Family::Name(FONT_MONO_FAMILY),
    weight: Weight::Medium,
    ..Font::DEFAULT
};

/// Sans font for reading text in a locale. Locales with a script pack name the pack family
/// explicitly, because in the browser cosmic-text has no locale-aware fallback and would pick
/// Han glyphs from whichever CJK pack loaded first.
#[must_use]
pub(crate) fn sans_for(locale: Locale, weight: Weight) -> Font {
    Font {
        family: Family::Name(locale.font_family()),
        weight,
        ..Font::DEFAULT
    }
}

/// Font for one string drawn by a custom widget with `Shaping::Auto`.
///
/// `Auto` shapes pure ASCII text without font fallback, and the Arabic and Indic script packs
/// carry no Latin letters, so an English label or an identifier drawn in such a face would show
/// missing glyphs. ASCII text therefore always uses the embedded Plex Sans family, keeping the
/// requested weight; any other text keeps `font` and is shaped with fallback.
#[must_use]
pub(crate) fn font_for_text(font: Font, content: &str) -> Font {
    if content.is_ascii() && font.family != Family::Name(FONT_MONO_FAMILY) {
        Font {
            family: Family::Name(FONT_SANS_FAMILY),
            ..font
        }
    } else {
        font
    }
}
