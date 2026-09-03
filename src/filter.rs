//! Allocation-bounded filtering and stable sorting over compact records.

use std::cmp::Ordering;
use std::sync::Arc;

use crate::model::{Dataset, LocationId, LocationKind, LocationRecord, SymbolId};

mod types;

pub use types::{
    FilterSet, FloatRange, OptionalFacet, OptionalNumeric, SortField, parse_optional_number,
};

/// Precomputed search strings and reusable index scanning logic.
pub struct FilterEngine {
    dataset: Arc<Dataset>,
    searchable: Vec<String>,
    folded_symbols: Vec<String>,
    name_order: Vec<LocationId>,
}

impl FilterEngine {
    /// Precomputes Unicode-lowercased name and identifier strings once at startup.
    #[must_use]
    pub fn new(dataset: Arc<Dataset>) -> Self {
        let folded_symbols: Vec<String> = dataset
            .stored
            .dictionary
            .iter()
            .map(|value| value.to_lowercase())
            .collect();
        let searchable = dataset
            .stored
            .locations
            .iter()
            .map(|record| {
                let name = dataset.symbol(record.name).unwrap_or_default();
                let key = dataset.symbol(record.key).unwrap_or_default();
                format!("{}\0{}", name.to_lowercase(), key.to_lowercase())
            })
            .collect();
        let mut name_order: Vec<LocationId> = dataset
            .stored
            .locations
            .iter()
            .map(|record| record.id)
            .collect();
        name_order.sort_by(|left, right| {
            let left_name = dataset
                .location(*left)
                .and_then(|record| symbol_key(&folded_symbols, record.name))
                .unwrap_or_default();
            let right_name = dataset
                .location(*right)
                .and_then(|record| symbol_key(&folded_symbols, record.name))
                .unwrap_or_default();
            left_name.cmp(right_name).then_with(|| left.cmp(right))
        });
        Self {
            dataset,
            searchable,
            folded_symbols,
            name_order,
        }
    }

    /// Scans compact records and returns only matching location IDs.
    #[must_use]
    pub fn apply(&self, filters: &FilterSet, sort: SortField, ascending: bool) -> Vec<LocationId> {
        let query = filters.search.to_lowercase();
        let mut ids: Vec<LocationId> = if sort == SortField::Name {
            let ordered: Box<dyn Iterator<Item = LocationId> + '_> = if ascending {
                Box::new(self.name_order.iter().copied())
            } else {
                Box::new(self.name_order.iter().rev().copied())
            };
            ordered
                .filter(|id| {
                    self.dataset
                        .location(*id)
                        .is_some_and(|record| self.matches(record, filters, &query))
                })
                .collect()
        } else {
            self.dataset
                .stored
                .locations
                .iter()
                .filter(|record| self.matches(record, filters, &query))
                .map(|record| record.id)
                .collect()
        };
        if sort != SortField::Name {
            ids.sort_by(|left, right| self.compare(*left, *right, sort, ascending));
        }
        ids
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

    fn compare(
        &self,
        left: LocationId,
        right: LocationId,
        field: SortField,
        ascending: bool,
    ) -> Ordering {
        let Some(left_record) = self.dataset.location(left) else {
            return Ordering::Equal;
        };
        let Some(right_record) = self.dataset.location(right) else {
            return Ordering::Equal;
        };
        let ordering = match field {
            SortField::Name => symbols(
                &self.folded_symbols,
                Some(left_record.name),
                Some(right_record.name),
            ),
            SortField::Identifier => symbols(
                &self.folded_symbols,
                Some(left_record.key),
                Some(right_record.key),
            ),
            SortField::Kind => left_record.kind.cmp(&right_record.kind),
            SortField::Topography => symbols(
                &self.folded_symbols,
                Some(left_record.topography),
                Some(right_record.topography),
            ),
            SortField::Vegetation => optional_symbols(
                &self.folded_symbols,
                left_record.vegetation,
                right_record.vegetation,
            ),
            SortField::Climate => optional_symbols(
                &self.folded_symbols,
                left_record.climate,
                right_record.climate,
            ),
            SortField::RiverLevel => optional_values(
                left_record.river.as_ref().map(|value| value.level.0),
                right_record.river.as_ref().map(|value| value.level.0),
            ),
        };
        let directed = if ascending {
            ordering
        } else {
            reverse_non_null(ordering, field, left_record, right_record)
        };
        directed.then_with(|| left.cmp(&right))
    }
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
        None => range.min.is_none() && range.max.is_none(),
    }
}

fn ranges_are_empty(first: FloatRange, second: FloatRange) -> bool {
    first.min.is_none() && first.max.is_none() && second.min.is_none() && second.max.is_none()
}

fn symbols(folded: &[String], left: Option<SymbolId>, right: Option<SymbolId>) -> Ordering {
    optional_values(
        left.and_then(|value| symbol_key(folded, value)),
        right.and_then(|value| symbol_key(folded, value)),
    )
}

fn optional_symbols(
    folded: &[String],
    left: Option<SymbolId>,
    right: Option<SymbolId>,
) -> Ordering {
    symbols(folded, left, right)
}

fn symbol_key(folded: &[String], symbol: SymbolId) -> Option<&str> {
    usize::try_from(symbol.0)
        .ok()
        .and_then(|index| folded.get(index))
        .map(String::as_str)
}

fn optional_values<T: Ord>(left: Option<T>, right: Option<T>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn reverse_non_null(
    ordering: Ordering,
    field: SortField,
    left: &LocationRecord,
    right: &LocationRecord,
) -> Ordering {
    let nullable = match field {
        SortField::Vegetation => (left.vegetation.is_none(), right.vegetation.is_none()),
        SortField::Climate => (left.climate.is_none(), right.climate.is_none()),
        SortField::RiverLevel => (left.river.is_none(), right.river.is_none()),
        _ => (false, false),
    };
    if nullable.0 != nullable.1 {
        ordering
    } else {
        ordering.reverse()
    }
}

#[cfg(test)]
mod tests;
