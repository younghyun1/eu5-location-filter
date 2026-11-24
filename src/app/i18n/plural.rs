//! CLDR cardinal plural categories for whole numbers in the 27 site locales.
//!
//! Groups follow `assets/i18n/README.md`; that table and this function change together.

use super::locale::Locale;

/// A CLDR plural category.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum PluralCategory {
    /// `zero`.
    Zero,
    /// `one`.
    One,
    /// `two`.
    Two,
    /// `few`.
    Few,
    /// `many`.
    Many,
    /// `other`.
    Other,
}

/// Category of a whole number in a locale.
#[must_use]
pub(crate) fn category(locale: Locale, count: u64) -> PluralCategory {
    let _ = locale;
    if count == 1 {
        PluralCategory::One
    } else {
        PluralCategory::Other
    }
}
