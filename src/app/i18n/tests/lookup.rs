//! Lookup, fallback and plural selection through `t` and `tp`.

use std::collections::BTreeSet;

use super::parity::placeholders;
use crate::app::i18n::catalog::{Entry, Forms, table};
use crate::app::i18n::{Locale, PluralCategory, Text, format, t, tp};

#[test]
fn text_indexes_and_keys_are_consistent() {
    let mut keys = BTreeSet::new();
    for (position, text) in Text::ALL.iter().enumerate() {
        assert_eq!(text.index(), position);
        assert!(keys.insert(text.key()), "duplicate key {}", text.key());
    }
    assert_eq!(keys.len(), Text::COUNT);
}

#[test]
fn locales_without_a_catalog_show_english() {
    for locale in Locale::ALL {
        if table(locale).is_some() {
            continue;
        }
        for text in Text::ALL {
            assert_eq!(t(locale, text), t(Locale::EnUs, text), "{}", locale.tag());
        }
    }
}

#[test]
fn english_plural_forms_select_by_count() {
    assert_eq!(
        tp(Locale::EnUs, Text::HeaderResultCount, 1),
        "{count} location"
    );
    assert_eq!(
        tp(Locale::EnUs, Text::HeaderResultCount, 0),
        "{count} locations"
    );
    assert_eq!(
        tp(Locale::EnUs, Text::HeaderResultCount, 2),
        "{count} locations"
    );
    assert_eq!(
        t(Locale::EnUs, Text::HeaderResultCount),
        "{count} locations"
    );
}

#[test]
fn counted_text_formats_with_its_count() {
    let template = tp(Locale::EnUs, Text::HeaderResultCount, 1);
    assert_eq!(format(template, &[("count", "1")]), "1 location");
}

#[test]
fn plain_keys_ignore_the_count() {
    assert_eq!(
        tp(Locale::EnUs, Text::AppTitle, 5),
        t(Locale::EnUs, Text::AppTitle)
    );
    assert_eq!(t(Locale::EnUs, Text::AppTitle), "EU5 Location DB");
}

#[test]
fn values_with_placeholders_keep_brace_syntax() {
    assert!(placeholders(t(Locale::EnUs, Text::HeaderBuild)).contains("version"));
}

#[test]
fn missing_forms_fall_back_to_other() {
    let forms = Forms {
        zero: None,
        one: Some("one"),
        two: None,
        few: None,
        many: None,
        other: "other",
    };
    assert_eq!(forms.select(PluralCategory::One), "one");
    assert_eq!(forms.select(PluralCategory::Few), "other");
    assert_eq!(forms.select(PluralCategory::Other), "other");
}

#[test]
fn every_table_entry_is_reachable_through_t() {
    for locale in Locale::ALL {
        let Some(entries) = table(locale) else {
            continue;
        };
        for (text, entry) in Text::ALL.iter().zip(entries.iter()) {
            let expected = match entry {
                Entry::Plain(value) => *value,
                Entry::Plural(forms) => forms.other,
            };
            assert_eq!(
                t(locale, *text),
                expected,
                "{} {}",
                locale.tag(),
                text.key()
            );
        }
    }
}
