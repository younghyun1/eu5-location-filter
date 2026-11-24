//! Interface localization: the 27 site locales, generated catalog tables with English fallback,
//! CLDR plural selection, placeholder formatting, invariant number formatting, and script font
//! packs.

mod catalog;
mod fonts;
mod format;
mod locale;
mod plural;
#[cfg(test)]
mod tests;

pub(crate) use catalog::{Text, t, tp};
pub(crate) use fonts::FontPack;
pub(crate) use format::{format, format_into, push_fixed, push_grouped};
pub(crate) use locale::Locale;
#[cfg(test)]
pub(crate) use plural::{PluralCategory, category};
