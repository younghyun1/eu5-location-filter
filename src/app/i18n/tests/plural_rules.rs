//! Plural categories of whole numbers at the boundaries of every locale's CLDR rule.

use crate::app::i18n::{Locale, PluralCategory, category};

use PluralCategory::{Few, Many, One, Other, Two, Zero};

/// Asserts each case for each locale.
fn expect(locales: &[Locale], cases: &[(u64, PluralCategory)]) {
    for locale in locales {
        for (count, expected) in cases {
            assert_eq!(
                category(*locale, *count),
                *expected,
                "{} with count {count}",
                locale.tag()
            );
        }
    }
}

#[test]
fn languages_without_plural_forms_always_use_other() {
    expect(
        &[
            Locale::KoKr,
            Locale::JaJp,
            Locale::ZhHans,
            Locale::ZhHant,
            Locale::ViVn,
            Locale::IdId,
        ],
        &[
            (0, Other),
            (1, Other),
            (2, Other),
            (21, Other),
            (1_000_000, Other),
        ],
    );
}

#[test]
fn one_only_for_exactly_one() {
    expect(
        &[
            Locale::EnUs,
            Locale::DeDe,
            Locale::NlNl,
            Locale::SvSe,
            Locale::UrPk,
            Locale::TrTr,
            Locale::TeIn,
        ],
        &[
            (0, Other),
            (1, One),
            (2, Other),
            (11, Other),
            (21, Other),
            (1_000_000, Other),
        ],
    );
}

#[test]
fn zero_and_one_are_one_in_hindi_bengali_and_persian() {
    expect(
        &[Locale::HiIn, Locale::BnBd, Locale::FaIr],
        &[(0, One), (1, One), (2, Other), (10, Other), (100, Other)],
    );
}

#[test]
fn filipino_depends_on_the_last_digit() {
    expect(
        &[Locale::TlPh],
        &[
            (0, One),
            (1, One),
            (2, One),
            (3, One),
            (4, Other),
            (5, One),
            (6, Other),
            (7, One),
            (8, One),
            (9, Other),
            (10, One),
            (11, One),
            (14, Other),
            (16, Other),
            (19, Other),
            (24, Other),
            (100, One),
        ],
    );
}

#[test]
fn french_and_portuguese_treat_zero_as_one_and_millions_as_many() {
    expect(
        &[Locale::FrFr, Locale::PtBr],
        &[
            (0, One),
            (1, One),
            (2, Other),
            (999_999, Other),
            (1_000_000, Many),
            (1_000_001, Other),
            (2_000_000, Many),
        ],
    );
}

#[test]
fn spanish_and_italian_have_one_only_for_one() {
    expect(
        &[Locale::EsEs, Locale::ItIt],
        &[
            (0, Other),
            (1, One),
            (2, Other),
            (999_999, Other),
            (1_000_000, Many),
            (1_500_000, Other),
            (3_000_000, Many),
        ],
    );
}

#[test]
fn romanian_uses_few_for_zero_and_teens() {
    expect(
        &[Locale::RoRo],
        &[
            (0, Few),
            (1, One),
            (2, Few),
            (19, Few),
            (20, Other),
            (99, Other),
            (100, Other),
            (101, Few),
            (119, Few),
            (120, Other),
            (201, Few),
        ],
    );
}

#[test]
fn serbo_croatian_has_no_many() {
    expect(
        &[Locale::Sh],
        &[
            (0, Other),
            (1, One),
            (2, Few),
            (4, Few),
            (5, Other),
            (11, Other),
            (12, Other),
            (14, Other),
            (21, One),
            (22, Few),
            (101, One),
            (111, Other),
            (112, Other),
        ],
    );
}

#[test]
fn russian_and_ukrainian_use_many_for_the_remaining_integers() {
    expect(
        &[Locale::RuRu, Locale::UkUa],
        &[
            (0, Many),
            (1, One),
            (2, Few),
            (4, Few),
            (5, Many),
            (11, Many),
            (12, Many),
            (14, Many),
            (15, Many),
            (21, One),
            (22, Few),
            (25, Many),
            (100, Many),
            (101, One),
            (111, Many),
            (112, Many),
            (122, Few),
        ],
    );
}

#[test]
fn polish_has_one_only_for_one() {
    expect(
        &[Locale::PlPl],
        &[
            (0, Many),
            (1, One),
            (2, Few),
            (4, Few),
            (5, Many),
            (11, Many),
            (12, Many),
            (14, Many),
            (21, Many),
            (22, Few),
            (25, Many),
            (101, Many),
            (102, Few),
            (112, Many),
        ],
    );
}

#[test]
fn arabic_uses_all_six_categories() {
    expect(
        &[Locale::Ar],
        &[
            (0, Zero),
            (1, One),
            (2, Two),
            (3, Few),
            (10, Few),
            (11, Many),
            (99, Many),
            (100, Other),
            (101, Other),
            (102, Other),
            (103, Few),
            (110, Few),
            (111, Many),
            (199, Many),
            (200, Other),
            (1_000, Other),
            (1_003, Few),
        ],
    );
}

#[test]
fn extreme_counts_do_not_overflow() {
    for locale in Locale::ALL {
        for count in [0, 1, u64::MAX - 1, u64::MAX] {
            let _ = category(locale, count);
        }
    }
}
