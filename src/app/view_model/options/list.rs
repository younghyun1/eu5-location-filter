//! One facet's options with a key index and folded-label search.

use memchr::memmem::Finder;

use crate::filter::fold_search;
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

impl FacetOption {
    /// Builds an option and folds its label once.
    #[must_use]
    pub(crate) fn new(value: OptionValue, key: &str, label: String) -> Self {
        let folded = fold_search(&label);
        Self {
            value,
            key: key.to_owned(),
            label,
            folded,
        }
    }
}

/// Options of one facet, with a key index for view-state resolution.
#[derive(Clone, Debug, Default)]
pub(crate) struct OptionList {
    /// Options in display order.
    options: Vec<FacetOption>,
    /// Indices into `options` sorted by `key` bytes, for binary search.
    by_key: Vec<u32>,
}

/// Answer for an index outside the table, which `FacetId::index` never produces.
pub(super) static EMPTY: OptionList = OptionList {
    options: Vec::new(),
    by_key: Vec::new(),
};

impl OptionList {
    /// Builds a list and its key index from options in display order.
    #[must_use]
    pub(crate) fn new(options: Vec<FacetOption>) -> Self {
        let mut by_key: Vec<u32> = (0..options.len())
            .filter_map(|index| u32::try_from(index).ok())
            .collect();
        by_key.sort_by(|left, right| key_at(&options, *left).cmp(key_at(&options, *right)));
        Self { options, by_key }
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
        let slot = self
            .by_key
            .binary_search_by(|index| key_at(&self.options, *index).cmp(key))
            .ok()?;
        let index = self.by_key.get(slot).copied()?;
        self.value(index)
    }

    /// Replaces `out` with the indices whose folded label contains `fold_search(query)`, in
    /// display order; an empty query matches every option. The query is folded once per call.
    pub(crate) fn filter_into(&self, query: &str, out: &mut Vec<u32>) {
        out.clear();
        let folded = fold_search(query);
        if folded.is_empty() {
            out.extend((0..self.options.len()).filter_map(|index| u32::try_from(index).ok()));
            return;
        }
        let finder = Finder::new(folded.as_bytes());
        out.extend(
            self.options
                .iter()
                .enumerate()
                .filter(|(_, option)| finder.find(option.folded.as_bytes()).is_some())
                .filter_map(|(index, _)| u32::try_from(index).ok()),
        );
    }
}

/// Wire key of an option by index; empty when out of range, which sorts first and never occurs.
fn key_at(options: &[FacetOption], index: u32) -> &str {
    usize::try_from(index)
        .ok()
        .and_then(|index| options.get(index))
        .map_or("", |option| option.key.as_str())
}
