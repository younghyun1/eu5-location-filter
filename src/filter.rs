//! Allocation-bounded filtering and precomputed stable sorting.

use std::collections::HashSet;
use std::sync::Arc;

use crate::model::{
    Dataset, LocationId, LocationKind, LocationRecord, SymbolId, is_food_producing,
};

mod index;
mod sort;
mod text;
mod types;

pub use index::StoredFilterIndex;
use sort::SortOrders;
pub(crate) use text::fold_search;
pub use types::{
    FilterSet, FloatRange, OptionalFacet, OptionalNumeric, SortField, parse_optional_number,
};

/// Precomputed search text and fixed sort indexes over one immutable dataset.
pub struct FilterEngine {
    dataset: Arc<Dataset>,
    searchable: Vec<String>,
    sort_orders: SortOrders,
    food_raw_materials: HashSet<SymbolId>,
}

impl FilterEngine {
    /// Builds normalized search strings and bounded orders for every sortable field.
    #[must_use]
    pub fn new(dataset: Arc<Dataset>) -> Self {
        let (folded_symbols, searchable) = search_data(&dataset);
        let sort_orders = SortOrders::new(&dataset, &folded_symbols);
        let food_raw_materials = food_raw_materials(&dataset);
        Self {
            dataset,
            searchable,
            sort_orders,
            food_raw_materials,
        }
    }

    /// Builds a deterministic payload for the committed filter-index bundle.
    #[must_use]
    pub fn build_stored_index(dataset: &Dataset) -> StoredFilterIndex {
        let (folded_symbols, searchable) = search_data(dataset);
        let sort_orders = SortOrders::new(dataset, &folded_symbols).into_stored();
        StoredFilterIndex {
            format_version: index::INDEX_FORMAT_VERSION,
            app_id: crate::model::EU5_APP_ID,
            build_id: dataset.stored.build_id,
            location_count: u32::try_from(dataset.stored.locations.len()).unwrap_or(u32::MAX),
            searchable,
            orders: sort_orders,
        }
    }

    /// Restores validated search and sort indexes without rebuilding them.
    pub fn from_stored_index(
        dataset: Arc<Dataset>,
        stored: StoredFilterIndex,
    ) -> Result<Self, crate::AppError> {
        stored.validate(&dataset)?;
        let food_raw_materials = food_raw_materials(&dataset);
        Ok(Self {
            dataset,
            searchable: stored.searchable,
            sort_orders: SortOrders::from_stored(stored.orders),
            food_raw_materials,
        })
    }

    /// Scans one pre-sorted index and returns only matching location IDs.
    #[must_use]
    pub fn apply(&self, filters: &FilterSet, sort: SortField, ascending: bool) -> Vec<LocationId> {
        let query = fold_search(&filters.search);
        let Some(order) = self.sort_orders.get(sort, ascending) else {
            return Vec::new();
        };
        order
            .iter()
            .copied()
            .filter(|id| {
                self.dataset
                    .location(*id)
                    .is_some_and(|record| self.matches(record, filters, &query))
            })
            .collect()
    }

    /// Preserves selection only while its ID remains in the filtered index.
    #[must_use]
    pub fn preserve_selection(
        selected: Option<LocationId>,
        visible: &[LocationId],
    ) -> Option<LocationId> {
        selected.filter(|id| visible.contains(id))
    }

    fn matches(&self, record: &LocationRecord, filters: &FilterSet, query: &str) -> bool {
        let searchable = usize::try_from(record.id.0)
            .ok()
            .and_then(|index| self.searchable.get(index))
            .map(String::as_str)
            .unwrap_or_default();
        (filters.show_impassable || record.kind != LocationKind::Impassable)
            && (query.is_empty() || searchable.contains(query))
            && (filters.kinds.is_empty() || filters.kinds.contains(&record.kind))
            && (filters.topographies.is_empty()
                || filters.topographies.contains(&record.topography))
            && (filters.vegetation.is_empty() || filters.vegetation.contains(&record.vegetation))
            && (filters.climates.is_empty() || filters.climates.contains(&record.climate))
            && facet_matches(filters.continent, Some(record.hierarchy.continent))
            && facet_matches(filters.subcontinent, Some(record.hierarchy.subcontinent))
            && facet_matches(filters.region, Some(record.hierarchy.region))
            && facet_matches(filters.area, Some(record.hierarchy.area))
            && facet_matches(filters.province, Some(record.hierarchy.province))
            && facet_matches(filters.religion, record.religion)
            && facet_matches(filters.culture, record.culture)
            && facet_matches(filters.raw_material, record.raw_material)
            && (!filters.food_producing_only
                || record
                    .raw_material
                    .is_some_and(|material| self.food_raw_materials.contains(&material)))
            && facet_matches(filters.modifier, record.modifier)
            && filters.rgb.is_none_or(|color| record.color == color)
            && filters.coastal.is_none_or(|value| record.coastal == value)
            && filters
                .river_presence
                .is_none_or(|value| record.river.is_some() == value)
            && optional_range_matches(
                record.river.as_ref().map(|river| f32::from(river.level.0)),
                FloatRange {
                    min: filters.river_level_min.map(f32::from),
                    max: filters.river_level_max.map(f32::from),
                },
            )
            && numeric_matches(
                filters.harbor_presence,
                filters.harbor_range,
                record.harbor_suitability,
            )
            && filters
                .movement_presence
                .is_none_or(|value| record.movement_assistance.is_some() == value)
            && match record.movement_assistance {
                Some(value) => {
                    filters.movement_x.matches(value[0]) && filters.movement_y.matches(value[1])
                }
                None => ranges_are_empty(filters.movement_x, filters.movement_y),
            }
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

fn food_raw_materials(dataset: &Dataset) -> HashSet<SymbolId> {
    dataset
        .stored
        .dictionary
        .iter()
        .enumerate()
        .filter(|(_, key)| is_food_producing(key))
        .filter_map(|(index, _)| u32::try_from(index).ok().map(SymbolId))
        .collect()
}

fn facet_matches(filter: OptionalFacet, value: Option<SymbolId>) -> bool {
    match filter {
        OptionalFacet::Any => true,
        OptionalFacet::Missing => value.is_none(),
        OptionalFacet::Value(expected) => value == Some(expected),
    }
}

fn numeric_matches(filter: OptionalNumeric, range: FloatRange, value: Option<f32>) -> bool {
    match (filter, value) {
        (OptionalNumeric::Any, None) => range.min.is_none() && range.max.is_none(),
        (OptionalNumeric::Missing, None) => true,
        (OptionalNumeric::Present, None) | (OptionalNumeric::Missing, Some(_)) => false,
        (OptionalNumeric::Any | OptionalNumeric::Present, Some(value)) => range.matches(value),
    }
}

fn optional_range_matches(value: Option<f32>, range: FloatRange) -> bool {
    match value {
        Some(value) => range.matches(value),
        None => range.min.is_none(),
    }
}

fn ranges_are_empty(first: FloatRange, second: FloatRange) -> bool {
    first.min.is_none() && first.max.is_none() && second.min.is_none() && second.max.is_none()
}

#[cfg(test)]
mod tests;
