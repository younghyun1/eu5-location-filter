//! Resolution of raw view-state values against the loaded dataset.

use super::decode::{RawItem, RawViewState};
use crate::app::view_model::{FacetId, FacetOptions, OptionValue, SortState, set_selected};
use crate::filter::FilterSet;
use crate::model::{Dataset, LocationId, LocationKind};

/// Location IDs sorted by key text, built once per dataset load, so a selected key resolves by
/// binary search without a per-dataset hash map of strings.
#[derive(Clone, Debug, Default)]
pub(crate) struct LocationKeys {
    sorted: Vec<LocationId>,
}

impl LocationKeys {
    /// Sorts every location by its key text.
    #[must_use]
    pub(crate) fn build(dataset: &Dataset) -> Self {
        let mut sorted: Vec<LocationId> = dataset
            .stored
            .locations
            .iter()
            .map(|record| record.id)
            .collect();
        sorted.sort_unstable_by(|left, right| key_of(dataset, *left).cmp(key_of(dataset, *right)));
        Self { sorted }
    }

    /// Location with this key.
    #[must_use]
    pub(crate) fn find(&self, dataset: &Dataset, key: &str) -> Option<LocationId> {
        match self
            .sorted
            .binary_search_by(|id| key_of(dataset, *id).cmp(key))
        {
            Ok(position) => self.sorted.get(position).copied(),
            Err(_) => None,
        }
    }
}

/// Key text of a location; empty for an ID outside the dataset, which `build` never stores.
fn key_of(dataset: &Dataset, id: LocationId) -> &str {
    match dataset.location(id).and_then(|record| dataset.symbol(record.key)) {
        Some(key) => key,
        None => "",
    }
}

/// A view state ready to replace the application's filter, sort and selection state.
#[derive(Clone, Debug)]
pub(crate) struct ResolvedView {
    /// Filters with every unknown item dropped.
    pub(crate) filters: FilterSet,
    /// Sort.
    pub(crate) sort: SortState,
    /// Selection candidate; update keeps it only if it is in the new results.
    pub(crate) selected: Option<LocationId>,
}

/// Resolves facet items through each facet's option list (dropping unknown items and missing
/// values where a facet has none) and the selected key through `keys`.
#[must_use]
pub(crate) fn resolve(
    raw: &RawViewState,
    options: &FacetOptions,
    keys: &LocationKeys,
    dataset: &Dataset,
) -> ResolvedView {
    let mut filters = FilterSet {
        search: raw.search.clone(),
        food_producing_only: raw.food_only,
        rgb: raw.rgb,
        harbor_range: raw.harbor_range,
        movement_x: raw.movement_x,
        movement_y: raw.movement_y,
        show_impassable: raw.show_impassable,
        ..FilterSet::default()
    };
    for facet in FacetId::ALL {
        let items = raw.facets.get(facet.index()).and_then(Option::as_ref);
        let applied = match items {
            Some(items) => apply_items(&mut filters, options, facet, items),
            None => 0,
        };
        // Type defaults to land only both when absent and when every listed kind was dropped;
        // only an empty list (`k.`) means every kind.
        let empty_list = matches!(items, Some(items) if items.is_empty());
        if facet == FacetId::Kind && applied == 0 && !empty_list {
            filters.kinds.insert(LocationKind::Land);
        }
    }
    let selected = match raw.selected.as_deref() {
        Some(key) => keys.find(dataset, key),
        None => None,
    };
    ResolvedView {
        filters,
        sort: raw.sort,
        selected,
    }
}

/// Selects every item that names an option of the facet and returns how many did.
fn apply_items(
    filters: &mut FilterSet,
    options: &FacetOptions,
    facet: FacetId,
    items: &[RawItem],
) -> usize {
    let list = options.get(facet);
    let mut applied = 0;
    for item in items {
        let value = match item {
            RawItem::Missing if facet.has_missing_option() => Some(OptionValue::Symbol(None)),
            RawItem::Missing => None,
            // An escaped `!` decodes to the text "!", which must not reach the missing option.
            RawItem::Key(key) => match list.find_key(key) {
                Some(OptionValue::Symbol(None)) | None => None,
                Some(value) => Some(value),
            },
        };
        if let Some(value) = value {
            // A repeated item changes nothing but still counts as named.
            let _ = set_selected(filters, facet, value, true);
            applied += 1;
        }
    }
    applied
}
