//! Original record scan retained only as a correctness and timing oracle.

use super::{FilterEngine, FilterSet, FloatRange, SortField, fold_search};
use crate::model::{
    Dataset, LocationId, LocationKind, LocationRecord, SymbolId, is_food_producing,
};
use std::collections::HashSet;

pub(super) struct ReferenceScan<'a> {
    searchable: &'a [String],
    food_raw_materials: &'a HashSet<SymbolId>,
}

impl FilterEngine {
    pub(super) fn apply_scan(
        &self,
        filters: &FilterSet,
        sort: SortField,
        ascending: bool,
    ) -> Vec<LocationId> {
        let query = fold_search(&filters.search);
        let reference = ReferenceScan {
            searchable: &self.searchable,
            food_raw_materials: &self.reference_food,
        };
        let Some(order) = self.sort_orders.get(sort, ascending) else {
            return Vec::new();
        };
        order
            .iter()
            .copied()
            .filter(|id| {
                self.dataset
                    .location(*id)
                    .is_some_and(|record| reference.matches(record, filters, &query))
            })
            .collect()
    }
}

impl ReferenceScan<'_> {
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
            && facet_matches(&filters.continents, Some(record.hierarchy.continent))
            && facet_matches(&filters.subcontinents, Some(record.hierarchy.subcontinent))
            && facet_matches(&filters.regions, Some(record.hierarchy.region))
            && facet_matches(&filters.areas, Some(record.hierarchy.area))
            && facet_matches(&filters.provinces, Some(record.hierarchy.province))
            && facet_matches(&filters.religions, record.religion)
            && facet_matches(&filters.cultures, record.culture)
            && facet_matches(&filters.raw_materials, record.raw_material)
            && (!filters.food_producing_only
                || record
                    .raw_material
                    .is_some_and(|material| self.food_raw_materials.contains(&material)))
            && facet_matches(&filters.modifiers, record.modifier)
            && filters.rgb.is_none_or(|color| record.color == color)
            && selection_matches(&filters.coastal, record.coastal)
            && selection_matches(&filters.river_presence, record.river.is_some())
            && river_range_matches(record.river.as_ref().map(|river| river.level.0), filters)
            && numeric_matches(
                &filters.harbor_presence,
                filters.harbor_range,
                record.harbor_suitability,
            )
            && selection_matches(
                &filters.movement_presence,
                record.movement_assistance.is_some(),
            )
            && match record.movement_assistance {
                Some(value) => {
                    filters.movement_x.matches(value[0]) && filters.movement_y.matches(value[1])
                }
                None => ranges_are_empty(filters.movement_x, filters.movement_y),
            }
    }
}

pub(super) fn food_raw_materials(dataset: &Dataset) -> HashSet<SymbolId> {
    dataset
        .stored
        .dictionary
        .iter()
        .enumerate()
        .filter(|(_, key)| is_food_producing(key))
        .filter_map(|(index, _)| u32::try_from(index).ok().map(SymbolId))
        .collect()
}

fn facet_matches(filter: &HashSet<Option<SymbolId>>, value: Option<SymbolId>) -> bool {
    filter.is_empty() || filter.contains(&value)
}

fn numeric_matches(filter: &HashSet<bool>, range: FloatRange, value: Option<f32>) -> bool {
    match value {
        Some(value) => selection_matches(filter, true) && range.matches(value),
        None => {
            (filter.contains(&false) || filter.is_empty())
                && range.min.is_none()
                && range.max.is_none()
        }
    }
}

fn selection_matches<T: Eq + std::hash::Hash>(filter: &HashSet<T>, value: T) -> bool {
    filter.is_empty() || filter.contains(&value)
}

fn river_range_matches(value: Option<u8>, filters: &FilterSet) -> bool {
    match value {
        Some(value) => {
            (filters.river_level_min.is_empty()
                || filters
                    .river_level_min
                    .iter()
                    .any(|minimum| value >= *minimum))
                && (filters.river_level_max.is_empty()
                    || filters
                        .river_level_max
                        .iter()
                        .any(|maximum| value <= *maximum))
        }
        None => filters.river_level_min.is_empty(),
    }
}

fn ranges_are_empty(first: FloatRange, second: FloatRange) -> bool {
    first.min.is_none() && first.max.is_none() && second.min.is_none() && second.max.is_none()
}
