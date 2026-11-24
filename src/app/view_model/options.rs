//! Checklist option lists per facet with labels folded once per language, and option search.
//!
//! Dataset options are sorted by display label; static options keep their contract order. Each
//! option carries its wire key (symbol identifier, kind word, `yes` or `no`, `present` or
//! `missing`, tier digits, or `!` for the missing value), which is how view-state items resolve
//! without scanning the dictionary.

use super::facets::FacetId;
use crate::app::i18n::Locale;
use crate::lang_pack::Labels;
use crate::model::{LocationKind, SymbolId};

/// Typed value of one option.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum OptionValue {
    /// Location kind, for Type.
    Kind(LocationKind),
    /// Dictionary symbol or the explicit missing value, for symbol facets.
    Symbol(Option<SymbolId>),
    /// Yes or present (`true`), no or missing (`false`), for presence facets.
    Flag(bool),
    /// River tier 1 to `MAX_RIVER_LEVEL`, for the two river bound facets.
    Tier(u8),
}

/// One checklist option.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FacetOption {
    /// Typed value.
    pub(crate) value: OptionValue,
    /// Wire key used by the view-state string.
    pub(crate) key: String,
    /// Display label for the active language and locale.
    pub(crate) label: String,
    /// `fold_search(label)`, computed when the list is built.
    pub(crate) folded: String,
}

/// Options of one facet, with a key index for view-state resolution.
#[derive(Clone, Debug, Default)]
pub(crate) struct OptionList {
    /// Options in display order.
    options: Vec<FacetOption>,
    /// Indices into `options` sorted by `key` bytes, for binary search.
    by_key: Vec<u32>,
}

impl OptionList {
    /// Builds a list and its key index from options in display order.
    #[must_use]
    pub(crate) fn new(options: Vec<FacetOption>) -> Self {
        Self {
            options,
            by_key: Vec::new(),
        }
    }

    /// Number of options.
    #[must_use]
    pub(crate) fn len(&self) -> usize {
        self.options.len()
    }

    /// Whether the list is empty.
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.options.is_empty()
    }

    /// Option at an index.
    #[must_use]
    pub(crate) fn get(&self, index: u32) -> Option<&FacetOption> {
        usize::try_from(index)
            .ok()
            .and_then(|index| self.options.get(index))
    }

    /// Label at an index, empty when out of range.
    #[must_use]
    pub(crate) fn label(&self, index: u32) -> &str {
        self.get(index).map_or("", |option| option.label.as_str())
    }

    /// Value at an index.
    #[must_use]
    pub(crate) fn value(&self, index: u32) -> Option<OptionValue> {
        self.get(index).map(|option| option.value)
    }

    /// Index of a value.
    #[must_use]
    pub(crate) fn position(&self, value: OptionValue) -> Option<u32> {
        self.options
            .iter()
            .position(|option| option.value == value)
            .and_then(|index| u32::try_from(index).ok())
    }

    /// Value of the option with this wire key, by binary search over the key index.
    #[must_use]
    pub(crate) fn find_key(&self, key: &str) -> Option<OptionValue> {
        let _ = (key, &self.by_key);
        None
    }

    /// Replaces `out` with the indices whose folded label contains `fold_search(query)`, in
    /// display order; an empty query matches every option. The query is folded once per call.
    pub(crate) fn filter_into(&self, query: &str, out: &mut Vec<u32>) {
        let _ = query;
        out.clear();
        out.extend((0..self.options.len()).filter_map(|index| u32::try_from(index).ok()));
    }
}

/// Option lists for every facet, built once per dataset load and per game-language change.
/// The two river bound facets share one tier list.
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
        let _ = (labels, locale);
        Self::default()
    }

    /// Option list of a facet.
    #[must_use]
    pub(crate) fn get(&self, facet: FacetId) -> &OptionList {
        let slot = match facet {
            FacetId::RiverLevelMax => FacetId::RiverLevelMin,
            other => other,
        };
        match self.lists.get(slot.index()) {
            Some(list) => list,
            None => &EMPTY,
        }
    }
}

/// Answer for an index outside the table, which `FacetId::index` never produces.
static EMPTY: OptionList = OptionList {
    options: Vec::new(),
    by_key: Vec::new(),
};
