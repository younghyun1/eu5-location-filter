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
///
/// Rules are the CLDR cardinal rules evaluated for integers (no visible fraction digits, so the
/// `v = 0` and `f` conditions always hold). The match lists every locale, so adding a locale
/// forces a decision here.
#[must_use]
pub(crate) fn category(locale: Locale, count: u64) -> PluralCategory {
    match locale {
        // No grammatical number.
        Locale::KoKr
        | Locale::JaJp
        | Locale::ZhHans
        | Locale::ZhHant
        | Locale::ViVn
        | Locale::IdId => PluralCategory::Other,
        // one: i = 1 and v = 0 (English, German, Dutch, Swedish, Urdu) or n = 1 (Turkish, Telugu).
        Locale::EnUs
        | Locale::DeDe
        | Locale::NlNl
        | Locale::SvSe
        | Locale::UrPk
        | Locale::TrTr
        | Locale::TeIn => one_if(count == 1),
        // one: i = 0 or n = 1.
        Locale::HiIn | Locale::BnBd | Locale::FaIr => one_if(count <= 1),
        // one: v = 0 and i % 10 != 4, 6, 9 (the one rule for 1, 2 and 3 is subsumed).
        Locale::TlPh => one_if(!matches!(count % 10, 4 | 6 | 9)),
        // one: i = 0, 1; many: a non-zero multiple of one million.
        Locale::FrFr | Locale::PtBr => {
            if count <= 1 {
                PluralCategory::One
            } else {
                millions(count)
            }
        }
        // one: n = 1 (es) or i = 1 and v = 0 (it); many: a non-zero multiple of one million.
        Locale::EsEs | Locale::ItIt => {
            if count == 1 {
                PluralCategory::One
            } else {
                millions(count)
            }
        }
        Locale::RoRo => romanian(count),
        Locale::Sh => south_slavic(count),
        Locale::RuRu | Locale::UkUa => east_slavic(count),
        Locale::PlPl => polish(count),
        Locale::Ar => arabic(count),
    }
}

fn one_if(condition: bool) -> PluralCategory {
    if condition {
        PluralCategory::One
    } else {
        PluralCategory::Other
    }
}

/// `many` for a non-zero multiple of one million (CLDR 42 for French, Spanish, Italian and
/// Portuguese), otherwise `other`.
fn millions(count: u64) -> PluralCategory {
    if count != 0 && count % 1_000_000 == 0 {
        PluralCategory::Many
    } else {
        PluralCategory::Other
    }
}

/// Romanian: one for 1; few for 0 and for n % 100 in 1..=19 otherwise; other.
fn romanian(count: u64) -> PluralCategory {
    if count == 1 {
        PluralCategory::One
    } else if count == 0 || (1..=19).contains(&(count % 100)) {
        PluralCategory::Few
    } else {
        PluralCategory::Other
    }
}

/// Serbo-Croatian (`sh`, as Serbian and Croatian): one for i % 10 = 1 except i % 100 = 11; few
/// for i % 10 in 2..=4 except i % 100 in 12..=14; other. There is no `many` for integers.
fn south_slavic(count: u64) -> PluralCategory {
    let (last, last_two) = (count % 10, count % 100);
    if last == 1 && last_two != 11 {
        PluralCategory::One
    } else if (2..=4).contains(&last) && !(12..=14).contains(&last_two) {
        PluralCategory::Few
    } else {
        PluralCategory::Other
    }
}

/// Russian and Ukrainian: as `south_slavic`, with `many` for the remaining integers.
fn east_slavic(count: u64) -> PluralCategory {
    match south_slavic(count) {
        PluralCategory::Other => PluralCategory::Many,
        category => category,
    }
}

/// Polish: one for 1 only; few as in Russian; many for every other integer.
fn polish(count: u64) -> PluralCategory {
    if count == 1 {
        return PluralCategory::One;
    }
    let (last, last_two) = (count % 10, count % 100);
    if (2..=4).contains(&last) && !(12..=14).contains(&last_two) {
        PluralCategory::Few
    } else {
        PluralCategory::Many
    }
}

/// Arabic: zero for 0, one for 1, two for 2, few for n % 100 in 3..=10, many for n % 100 in
/// 11..=99, other otherwise (100, 101, 102, 200 and so on).
fn arabic(count: u64) -> PluralCategory {
    match count {
        0 => PluralCategory::Zero,
        1 => PluralCategory::One,
        2 => PluralCategory::Two,
        _ => match count % 100 {
            3..=10 => PluralCategory::Few,
            11..=99 => PluralCategory::Many,
            _ => PluralCategory::Other,
        },
    }
}
