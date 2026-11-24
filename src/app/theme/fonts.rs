//! Font values for the embedded IBM Plex faces and the per-locale script packs.
//!
//! Only `Weight::Normal`, `Medium` and `Semibold` and the normal style and stretch are used: a
//! request that no loaded face matches in style or stretch can leave cosmic-text without any
//! candidate face.

use std::sync::atomic::{AtomicU32, Ordering};

use iced::Font;
use iced::advanced::text::LineHeight;
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

/// Number of font packs registered since start. Bounded by the number of packs.
static FONT_EPOCH: AtomicU32 = AtomicU32::new(0);

/// Records that a font pack was registered with the renderer.
pub(crate) fn note_font_registered() {
    let _ = FONT_EPOCH.fetch_add(1, Ordering::Relaxed);
}

/// Line height for text that custom widgets draw through the renderer's text cache.
///
/// That cache keys shaped text by content, font and metrics but not by which fonts are loaded,
/// so a label first drawn before its script font arrived would keep its missing glyphs for as
/// long as it stays on screen. Folding the registration count into the line height by an
/// invisible amount (a few millionths per pack) gives every cached entry a new key once a pack
/// loads, and the stale entries are trimmed on the next frame.
#[must_use]
pub(crate) fn cached_line_height(relative: f32) -> LineHeight {
    let epoch = FONT_EPOCH.load(Ordering::Relaxed).min(64);
    #[expect(
        clippy::cast_precision_loss,
        reason = "the epoch is at most 64, far inside f32's exact integer range"
    )]
    let nudge = epoch as f32 * 4.0e-6;
    LineHeight::Relative(relative + nudge)
}
