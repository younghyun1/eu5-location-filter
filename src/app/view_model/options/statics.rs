//! Option lists whose values the catalog and the data model fix: kinds, flags and river tiers.

use super::list::{FacetOption, OptionList, OptionValue};
use super::texts::{kind_text, river_tier_text};
use crate::app::i18n::{Locale, Text, format, t};
use crate::model::{LocationKind, MAX_RIVER_LEVEL};

/// Kinds in contract order with their wire words.
const KINDS: [(LocationKind, &str); 5] = [
    (LocationKind::Land, "land"),
    (LocationKind::Sea, "sea"),
    (LocationKind::Lake, "lake"),
    (LocationKind::Impassable, "impassable"),
    (LocationKind::Unknown, "unknown"),
];

/// Type options.
#[must_use]
pub(super) fn kinds(locale: Locale) -> OptionList {
    OptionList::new(
        KINDS
            .into_iter()
            .map(|(kind, key)| {
                FacetOption::new(
                    OptionValue::Kind(kind),
                    key,
                    t(locale, kind_text(kind)).to_owned(),
                )
            })
            .collect(),
    )
}

/// A two-option presence list: the `true` option first, then the `false` option.
#[must_use]
pub(super) fn flags(
    locale: Locale,
    on: (Text, &str),
    off: (Text, &str),
) -> OptionList {
    OptionList::new(vec![
        FacetOption::new(OptionValue::Flag(true), on.1, t(locale, on.0).to_owned()),
        FacetOption::new(OptionValue::Flag(false), off.1, t(locale, off.0).to_owned()),
    ])
}

/// River tiers 1 to `MAX_RIVER_LEVEL`, labelled with the tier name and capacity bonus.
#[must_use]
pub(super) fn tiers(locale: Locale) -> OptionList {
    let template = t(locale, Text::OptionRiverTier);
    OptionList::new(
        (1..=MAX_RIVER_LEVEL)
            .map(|level| {
                let level_text = level.to_string();
                let percent_text = u16::from(level).saturating_mul(10).to_string();
                let label = format(
                    template,
                    &[
                        ("level", level_text.as_str()),
                        ("tier", t(locale, river_tier_text(level))),
                        ("percent", percent_text.as_str()),
                    ],
                );
                FacetOption::new(OptionValue::Tier(level), &level_text, label)
            })
            .collect(),
    )
}
