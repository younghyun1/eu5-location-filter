//! Allocation-bounded queries over bundled facet, numeric, search, and sort indexes.

use crate::model::{Dataset, LocationId};
use std::sync::Arc;

mod bitmap;
mod facets;
mod index;
#[cfg(test)]
mod indexed_tests;
mod numeric;
mod posting;
mod query;
#[cfg(test)]
mod reference;
mod search;
mod sort;
mod sort_compare;
#[cfg(test)]
mod test_fixture;
#[cfg(test)]
mod tests;
mod text;
#[cfg(all(test, feature = "desktop"))]
mod timings;
mod types;

pub use index::StoredFilterIndex;
use sort::SortOrders;
pub(crate) use text::fold_search;
pub use types::{FilterSet, FloatRange, SortField, parse_optional_number};

/// Immutable precomputed indexes; queries allocate only bounded masks and result IDs.
pub struct FilterEngine {
    dataset: Arc<Dataset>,
    searchable: Vec<String>,
    sort_orders: SortOrders,
    query_index: query::QueryIndex,
    #[cfg(test)]
    reference_food: std::collections::HashSet<crate::model::SymbolId>,
}

impl FilterEngine {
    /// Builds indexes for an external dataset or a synthetic test fixture.
    #[must_use]
    pub fn new(dataset: Arc<Dataset>) -> Self {
        let stored = Self::build_stored_index(&dataset);
        Self::restore(dataset, stored)
    }

    /// Builds the deterministic payload offline, without accessing the game installation.
    #[must_use]
    pub fn build_stored_index(dataset: &Dataset) -> StoredFilterIndex {
        let (folded_symbols, searchable) = search_data(dataset);
        let query = query::QueryIndex::build(dataset, &searchable);
        StoredFilterIndex {
            format_version: index::INDEX_FORMAT_VERSION,
            app_id: crate::model::EU5_APP_ID,
            build_id: dataset.stored.build_id,
            location_count: u32::try_from(dataset.stored.locations.len()).unwrap_or(u32::MAX),
            searchable,
            orders: SortOrders::new(dataset, &folded_symbols).into_stored(),
            query,
        }
    }

    /// Restores validated indexes without runtime index generation or sorting.
    pub fn from_stored_index(
        dataset: Arc<Dataset>,
        stored: StoredFilterIndex,
    ) -> Result<Self, crate::AppError> {
        stored.validate(&dataset)?;
        Ok(Self::restore(dataset, stored))
    }

    fn restore(dataset: Arc<Dataset>, stored: StoredFilterIndex) -> Self {
        Self {
            #[cfg(test)]
            reference_food: reference::food_raw_materials(&dataset),
            dataset,
            searchable: stored.searchable,
            sort_orders: SortOrders::from_stored(stored.orders),
            query_index: stored.query,
        }
    }

    /// Intersects indexed criteria, verifies search candidates, and orders matching IDs.
    #[must_use]
    pub fn apply(&self, filters: &FilterSet, sort: SortField, ascending: bool) -> Vec<LocationId> {
        let mask = self
            .query_index
            .apply(&self.dataset, &self.searchable, filters);
        self.sort_orders.select(&mask, sort, ascending)
    }

    /// Preserves selection only while its ID remains in the filtered index.
    #[must_use]
    pub fn preserve_selection(
        selected: Option<LocationId>,
        visible: &[LocationId],
    ) -> Option<LocationId> {
        selected.filter(|id| visible.contains(id))
    }
}

fn search_data(dataset: &Dataset) -> (Vec<String>, Vec<String>) {
    let folded_symbols = dataset
        .stored
        .dictionary
        .iter()
        .map(|value| text::fold_sort(value))
        .collect();
    let searchable = dataset
        .stored
        .locations
        .iter()
        .map(|record| {
            let name = dataset.symbol(record.name).unwrap_or_default();
            let key = dataset.symbol(record.key).unwrap_or_default();
            format!("{}\0{}", fold_search(name), fold_search(key))
        })
        .collect();
    (folded_symbols, searchable)
}
