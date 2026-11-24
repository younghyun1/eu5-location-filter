//! Interface text tables generated from `assets/i18n/*.txt`, and lookup with English fallback.
//!
//! `build.rs` writes `$OUT_DIR/i18n_catalog.rs`, included below, which defines exactly:
//!
//! - `pub enum Text` with one variant per base key of `en-US.txt`, in file order, named by
//!   splitting the key at `.` and `_` and joining the capitalized parts (`grid.show_details` is
//!   `Text::GridShowDetails`), each with a `///` line holding the key; deriving `Clone, Copy,
//!   Debug, Eq, Hash, Ord, PartialEq, PartialOrd`.
//! - `impl Text` with `pub const COUNT: usize`, `pub const ALL: [Text; Text::COUNT]`,
//!   `pub const fn key(self) -> &'static str` (the dotted key) and `pub const fn index(self) ->
//!   usize` (position in `ALL`).
//! - `static CATALOGS: [Option<&'static [Entry; Text::COUNT]>; Locale::COUNT]` in
//!   `Locale::ALL` order. English is always `Some`; a locale without a catalog file is `None`.
//!
//! Values have backslash escapes decoded. A value without placeholders also has `{{` and `}}`
//! decoded, so `t` returns display text; a value with placeholders keeps brace syntax for
//! `format_into`.

use super::locale::Locale;
use super::plural::{PluralCategory, category};

/// One catalog value.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Entry {
    /// A string without plural forms.
    Plain(&'static str),
    /// A counted string.
    Plural(&'static Forms),
}

/// Plural forms of one key in one locale; only the categories the locale uses are present.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Forms {
    /// `zero` form.
    pub(crate) zero: Option<&'static str>,
    /// `one` form.
    pub(crate) one: Option<&'static str>,
    /// `two` form.
    pub(crate) two: Option<&'static str>,
    /// `few` form.
    pub(crate) few: Option<&'static str>,
    /// `many` form.
    pub(crate) many: Option<&'static str>,
    /// `other` form, always present.
    pub(crate) other: &'static str,
}

impl Forms {
    /// Form for a category, falling back to `other`.
    #[must_use]
    pub(crate) fn select(&self, category: PluralCategory) -> &'static str {
        let form = match category {
            PluralCategory::Zero => self.zero,
            PluralCategory::One => self.one,
            PluralCategory::Two => self.two,
            PluralCategory::Few => self.few,
            PluralCategory::Many => self.many,
            PluralCategory::Other => None,
        };
        form.unwrap_or(self.other)
    }
}

include!(concat!(env!("OUT_DIR"), "/i18n_catalog.rs"));

/// Catalog value for a locale, falling back to English; the `other` form for counted keys.
#[must_use]
pub(crate) fn t(locale: Locale, text: Text) -> &'static str {
    match entry(locale, text) {
        Some(Entry::Plain(value)) => value,
        Some(Entry::Plural(forms)) => forms.other,
        None => "",
    }
}

/// Plural form of a counted key for `count`; still a template holding `{count}`.
#[must_use]
pub(crate) fn tp(locale: Locale, text: Text, count: u64) -> &'static str {
    match entry(locale, text) {
        Some(Entry::Plain(value)) => value,
        Some(Entry::Plural(forms)) => forms.select(category(locale, count)),
        None => "",
    }
}

/// Catalog table of a locale without the English fallback, `None` for a locale with no file;
/// the parity tests need to tell a missing catalog from an English one.
#[cfg(test)]
pub(super) fn table(locale: Locale) -> Option<&'static [Entry; Text::COUNT]> {
    CATALOGS.get(locale.index()).copied().flatten()
}

fn entry(locale: Locale, text: Text) -> Option<Entry> {
    let table = CATALOGS
        .get(locale.index())
        .copied()
        .flatten()
        .or_else(|| CATALOGS.first().copied().flatten())?;
    table.get(text.index()).copied()
}
