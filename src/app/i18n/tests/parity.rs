//! Parity of the generated tables: every locale with a catalog has every English key with the
//! same placeholder names and exactly the plural forms its rule needs.

use std::collections::BTreeSet;

use crate::app::i18n::catalog::{Entry, Forms, table};
use crate::app::i18n::{Locale, PluralCategory, Text, category};

use PluralCategory::{Few, Many, One, Other, Two, Zero};

/// Placeholder names in a catalog value. Lenient on purpose: a stray brace is ignored here,
/// because the build script already rejects malformed values.
pub(super) fn placeholders(text: &str) -> BTreeSet<&str> {
    let mut names = BTreeSet::new();
    let bytes = text.as_bytes();
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        if byte == b'{' && bytes.get(index + 1) == Some(&b'{') {
            index += 2;
        } else if byte == b'{' {
            let rest = text.get(index + 1..).unwrap_or_default();
            match rest.find('}') {
                Some(length) => {
                    names.insert(rest.get(..length).unwrap_or_default());
                    index += length + 2;
                }
                None => index += 1,
            }
        } else {
            index += 1;
        }
    }
    names
}

/// The forms of a counted entry as (category, text) pairs in CLDR order.
fn forms_of(forms: &Forms) -> Vec<(PluralCategory, &'static str)> {
    let optional = [
        (Zero, forms.zero),
        (One, forms.one),
        (Two, forms.two),
        (Few, forms.few),
        (Many, forms.many),
    ];
    let mut present: Vec<(PluralCategory, &'static str)> = optional
        .into_iter()
        .filter_map(|(category, form)| form.map(|text| (category, text)))
        .collect();
    present.push((Other, forms.other));
    present
}

/// Categories the selector can return for whole numbers, plus `other`, which every counted key
/// must carry even where integers never select it.
fn produced_categories(locale: Locale) -> BTreeSet<u8> {
    let mut counts: Vec<u64> = (0..=1_200).collect();
    counts.extend([1_000_000, 1_000_001, 2_000_000, 1_000_000_000, u64::MAX]);
    let mut set: BTreeSet<u8> = counts
        .into_iter()
        .map(|count| rank(category(locale, count)))
        .collect();
    set.insert(rank(Other));
    set
}

/// Categories per locale as the README table lists them.
fn documented(locale: Locale) -> &'static [PluralCategory] {
    match locale {
        Locale::KoKr
        | Locale::JaJp
        | Locale::ZhHans
        | Locale::ZhHant
        | Locale::ViVn
        | Locale::IdId => &[Other],
        Locale::EnUs
        | Locale::DeDe
        | Locale::NlNl
        | Locale::SvSe
        | Locale::TrTr
        | Locale::HiIn
        | Locale::BnBd
        | Locale::TeIn
        | Locale::UrPk
        | Locale::FaIr
        | Locale::TlPh => &[One, Other],
        Locale::FrFr | Locale::EsEs | Locale::ItIt | Locale::PtBr => &[One, Many, Other],
        Locale::RoRo | Locale::Sh => &[One, Few, Other],
        Locale::RuRu | Locale::UkUa | Locale::PlPl => &[One, Few, Many, Other],
        Locale::Ar => &[Zero, One, Two, Few, Many, Other],
    }
}

/// Orderable stand-in for a category, which has no `Ord`.
fn rank(category: PluralCategory) -> u8 {
    match category {
        Zero => 0,
        One => 1,
        Two => 2,
        Few => 3,
        Many => 4,
        Other => 5,
    }
}

#[test]
fn english_catalog_is_complete() {
    assert!(table(Locale::EnUs).is_some());
    assert_eq!(Text::ALL.len(), Text::COUNT);
}

#[test]
fn selector_matches_the_documented_categories() {
    for locale in Locale::ALL {
        let documented: BTreeSet<u8> = documented(locale).iter().map(|c| rank(*c)).collect();
        assert_eq!(produced_categories(locale), documented, "{}", locale.tag());
    }
}

#[test]
fn every_locale_has_every_key_with_matching_placeholders() -> Result<(), String> {
    let english = table(Locale::EnUs).ok_or("the English catalog is missing")?;
    let mut checked = 0;
    for locale in Locale::ALL {
        let Some(entries) = table(locale) else {
            continue;
        };
        checked += 1;
        for (text, (entry, reference)) in Text::ALL.iter().zip(entries.iter().zip(english.iter())) {
            let context = format!("{} {}", locale.tag(), text.key());
            match (entry, reference) {
                (Entry::Plain(value), Entry::Plain(expected)) => {
                    assert_eq!(placeholders(value), placeholders(expected), "{context}");
                }
                (Entry::Plural(forms), Entry::Plural(expected)) => {
                    let want = placeholders(expected.other);
                    for (category, form) in forms_of(forms) {
                        let got = placeholders(form);
                        assert!(
                            got.is_subset(&want),
                            "{context} [{category:?}] adds a placeholder"
                        );
                        if category == Other {
                            assert_eq!(got, want, "{context} other form");
                        }
                    }
                }
                _ => return Err(format!("{context} differs in kind from English")),
            }
        }
    }
    assert!(checked >= 1);
    Ok(())
}

#[test]
fn counted_keys_carry_exactly_the_forms_the_rule_needs() {
    for locale in Locale::ALL {
        let Some(entries) = table(locale) else {
            continue;
        };
        let required = produced_categories(locale);
        for (text, entry) in Text::ALL.iter().zip(entries.iter()) {
            if let Entry::Plural(forms) = entry {
                let present: BTreeSet<u8> = forms_of(forms).iter().map(|(c, _)| rank(*c)).collect();
                assert_eq!(present, required, "{} {}", locale.tag(), text.key());
            }
        }
    }
}
