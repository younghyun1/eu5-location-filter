//! Option lists for every facet, built once per dataset load and per language change.

use super::list::{EMPTY, FacetOption, OptionList, OptionValue};
use super::statics::{flags, kinds, tiers};
use crate::app::i18n::{Locale, Text, format, t};
use crate::app::view_model::FacetId;
use crate::lang_pack::Labels;
use crate::model::{LocationRecord, SymbolId};

/// Option lists for every facet. The two river bound facets share one tier list.
#[derive(Clone, Debug, Default)]
pub(crate) struct FacetOptions {
    /// Lists indexed by `FacetId::index`; `RiverLevelMax` is answered from the
    /// `RiverLevelMin` slot, whose list it shares.
    lists: [OptionList; FacetId::COUNT],
}

impl FacetOptions {
    /// Builds every list: static options from the catalog for `locale`, dataset options from
    /// the symbols the records use, labelled through `labels` (`option.symbol_with_key` when the
    /// label differs from the key, the plain label for raw materials, `option.no_value` first
    /// where the facet has a missing option).
    #[must_use]
    pub(crate) fn build(labels: Labels<'_>, locale: Locale) -> Self {
        Self {
            lists: FacetId::ALL.map(|facet| build_list(labels, locale, facet)),
        }
    }

    /// Option list of a facet.
    #[must_use]
    pub(crate) fn get(&self, facet: FacetId) -> &OptionList {
        let slot = match facet {
            FacetId::RiverLevelMax => FacetId::RiverLevelMin,
            other => other,
        };
        self.lists.get(slot.index()).unwrap_or(&EMPTY)
    }
}

/// One facet's list; the shared tier list lives in the `RiverLevelMin` slot only.
fn build_list(labels: Labels<'_>, locale: Locale, facet: FacetId) -> OptionList {
    match facet {
        FacetId::Kind => kinds(locale),
        FacetId::Coastal => flags(locale, (Text::OptionYes, "yes"), (Text::OptionNo, "no")),
        FacetId::RiverPresence | FacetId::HarborPresence | FacetId::MovementPresence => flags(
            locale,
            (Text::OptionPresent, "present"),
            (Text::OptionMissing, "missing"),
        ),
        FacetId::RiverLevelMin => tiers(locale),
        FacetId::RiverLevelMax => OptionList::default(),
        FacetId::Continent
        | FacetId::Subcontinent
        | FacetId::Region
        | FacetId::Area
        | FacetId::Province
        | FacetId::Topography
        | FacetId::Vegetation
        | FacetId::Climate
        | FacetId::Religion
        | FacetId::Culture
        | FacetId::RawMaterial
        | FacetId::Modifier => symbol_list(labels, locale, facet),
    }
}

/// Symbol a record contributes to a dataset facet; other facets contribute none.
fn symbol_of(facet: FacetId, record: &LocationRecord) -> Option<SymbolId> {
    match facet {
        FacetId::Continent => Some(record.hierarchy.continent),
        FacetId::Subcontinent => Some(record.hierarchy.subcontinent),
        FacetId::Region => Some(record.hierarchy.region),
        FacetId::Area => Some(record.hierarchy.area),
        FacetId::Province => Some(record.hierarchy.province),
        FacetId::Topography => Some(record.topography),
        FacetId::Vegetation => record.vegetation,
        FacetId::Climate => record.climate,
        FacetId::Religion => record.religion,
        FacetId::Culture => record.culture,
        FacetId::RawMaterial => record.raw_material,
        FacetId::Modifier => record.modifier,
        FacetId::Kind
        | FacetId::Coastal
        | FacetId::RiverPresence
        | FacetId::RiverLevelMin
        | FacetId::RiverLevelMax
        | FacetId::HarborPresence
        | FacetId::MovementPresence => None,
    }
}

/// Options from the symbols the records use, sorted by folded display text, with the missing
/// value first where the facet has one.
fn symbol_list(labels: Labels<'_>, locale: Locale, facet: FacetId) -> OptionList {
    let dataset = labels.dataset();
    let mut symbols: Vec<SymbolId> = dataset
        .stored
        .locations
        .iter()
        .filter_map(|record| symbol_of(facet, record))
        .collect();
    symbols.sort_unstable();
    symbols.dedup();
    let template = t(locale, Text::OptionSymbolWithKey);
    let mut entries: Vec<FacetOption> = symbols
        .into_iter()
        .filter_map(|symbol| {
            let key = dataset.symbol(symbol)?;
            let label = labels.label(symbol).unwrap_or(key);
            let display = if facet == FacetId::RawMaterial {
                label.to_owned()
            } else if label == key {
                key.to_owned()
            } else {
                format(template, &[("label", label), ("key", key)])
            };
            Some(FacetOption::new(
                OptionValue::Symbol(Some(symbol)),
                key,
                display,
            ))
        })
        .collect();
    entries.sort_by(|left, right| {
        left.folded
            .cmp(&right.folded)
            .then_with(|| left.label.cmp(&right.label))
            .then_with(|| left.key.cmp(&right.key))
    });
    let mut options = Vec::with_capacity(entries.len().saturating_add(1));
    if facet.has_missing_option() {
        options.push(FacetOption::new(
            OptionValue::Symbol(None),
            "!",
            t(locale, Text::OptionNoValue).to_owned(),
        ));
    }
    options.extend(entries);
    OptionList::new(options)
}
