//! The locale tag list and the plural categories each locale must supply.

use super::model::Category;

/// Generated catalog file, included by `src/app/i18n/catalog.rs`.
pub const CATALOG_FILE: &str = "i18n_catalog.rs";

/// Site locale tags in `Locale::ALL` order (`src/app/i18n/locale.rs`); the generated
/// `CATALOGS` table uses this order, so both lists must change together.
pub const LOCALE_TAGS: [&str; 27] = [
    "en-US", "ko-KR", "fr-FR", "es-ES", "zh-Hans", "zh-Hant", "ja-JP", "de-DE", "ar", "hi-IN",
    "bn-BD", "te-IN", "pt-BR", "id-ID", "ru-RU", "ur-PK", "vi-VN", "tl-PH", "fa-IR", "it-IT",
    "pl-PL", "sv-SE", "sh", "ro-RO", "nl-NL", "uk-UA", "tr-TR",
];

/// Cardinal plural categories a locale supplies, per the table in `assets/i18n/README.md`.
/// This must agree with `src/app/i18n/plural.rs`; a unit test there checks that every category
/// the selector can return is listed here and the reverse.
pub fn required_categories(tag: &str) -> Option<&'static [Category]> {
    use Category::{Few, Many, One, Other, Two, Zero};
    let categories: &'static [Category] = match tag {
        "ko-KR" | "ja-JP" | "zh-Hans" | "zh-Hant" | "vi-VN" | "id-ID" => &[Other],
        "en-US" | "de-DE" | "nl-NL" | "sv-SE" | "tr-TR" | "hi-IN" | "bn-BD" | "te-IN"
        | "ur-PK" | "fa-IR" | "tl-PH" => &[One, Other],
        "fr-FR" | "es-ES" | "it-IT" | "pt-BR" => &[One, Many, Other],
        "ro-RO" | "sh" => &[One, Few, Other],
        "ru-RU" | "uk-UA" | "pl-PL" => &[One, Few, Many, Other],
        "ar" => &[Zero, One, Two, Few, Many, Other],
        _ => return None,
    };
    Some(categories)
}
