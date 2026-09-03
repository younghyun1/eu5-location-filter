//! Precomputed bounded sort orders for constant-time sort selection.

use std::cmp::Ordering;
use std::collections::HashMap;

use super::SortField;
use super::index::StoredSortOrder;
use crate::model::{Dataset, LocationId, LocationRecord, SymbolId};

struct DirectionOrders {
    ascending: Vec<LocationId>,
    descending: Vec<LocationId>,
}

pub(super) struct SortOrders {
    values: HashMap<SortField, DirectionOrders>,
}

impl SortOrders {
    pub(super) fn new(dataset: &Dataset, folded_symbols: &[String]) -> Self {
        let base: Vec<LocationId> = dataset
            .stored
            .locations
            .iter()
            .map(|record| record.id)
            .collect();
        let mut values = HashMap::with_capacity(SortField::ALL.len());
        for field in SortField::ALL {
            let mut ascending = base.clone();
            ascending.sort_by(|left, right| {
                compare(dataset, folded_symbols, *left, *right, field, true)
            });
            let mut descending = base.clone();
            descending.sort_by(|left, right| {
                compare(dataset, folded_symbols, *left, *right, field, false)
            });
            values.insert(
                field,
                DirectionOrders {
                    ascending,
                    descending,
                },
            );
        }
        Self { values }
    }

    pub(super) fn get(&self, field: SortField, ascending: bool) -> Option<&[LocationId]> {
        self.values.get(&field).map(|orders| {
            if ascending {
                orders.ascending.as_slice()
            } else {
                orders.descending.as_slice()
            }
        })
    }

    pub(super) fn into_stored(mut self) -> Vec<StoredSortOrder> {
        let mut stored = Vec::with_capacity(SortField::ALL.len());
        for field in SortField::ALL {
            let Some(orders) = self.values.remove(&field) else {
                continue;
            };
            stored.push(StoredSortOrder {
                field,
                ascending: orders.ascending,
                descending: orders.descending,
            });
        }
        stored
    }

    pub(super) fn from_stored(stored: Vec<StoredSortOrder>) -> Self {
        let values = stored
            .into_iter()
            .map(|orders| {
                (
                    orders.field,
                    DirectionOrders {
                        ascending: orders.ascending,
                        descending: orders.descending,
                    },
                )
            })
            .collect();
        Self { values }
    }
}

fn compare(
    dataset: &Dataset,
    folded: &[String],
    left: LocationId,
    right: LocationId,
    field: SortField,
    ascending: bool,
) -> Ordering {
    let Some(left_record) = dataset.location(left) else {
        return Ordering::Equal;
    };
    let Some(right_record) = dataset.location(right) else {
        return Ordering::Equal;
    };
    let ordering = compare_records(folded, left_record, right_record, field);
    let directed = if ascending {
        ordering
    } else {
        reverse_non_null(ordering, field, left_record, right_record)
    };
    directed.then_with(|| left.cmp(&right))
}

fn compare_records(
    folded: &[String],
    left: &LocationRecord,
    right: &LocationRecord,
    field: SortField,
) -> Ordering {
    match field {
        SortField::Color => left.color.cmp(&right.color),
        SortField::Name => symbols(folded, Some(left.name), Some(right.name)),
        SortField::Identifier => symbols(folded, Some(left.key), Some(right.key)),
        SortField::Kind => left.kind.cmp(&right.kind),
        SortField::Topography => symbols(folded, Some(left.topography), Some(right.topography)),
        SortField::Vegetation => symbols(folded, left.vegetation, right.vegetation),
        SortField::Climate => symbols(folded, left.climate, right.climate),
        SortField::Continent => symbols(
            folded,
            Some(left.hierarchy.continent),
            Some(right.hierarchy.continent),
        ),
        SortField::Subcontinent => symbols(
            folded,
            Some(left.hierarchy.subcontinent),
            Some(right.hierarchy.subcontinent),
        ),
        SortField::Region => symbols(
            folded,
            Some(left.hierarchy.region),
            Some(right.hierarchy.region),
        ),
        SortField::Area => symbols(
            folded,
            Some(left.hierarchy.area),
            Some(right.hierarchy.area),
        ),
        SortField::Province => symbols(
            folded,
            Some(left.hierarchy.province),
            Some(right.hierarchy.province),
        ),
        SortField::Religion => symbols(folded, left.religion, right.religion),
        SortField::Culture => symbols(folded, left.culture, right.culture),
        SortField::RawMaterial => symbols(folded, left.raw_material, right.raw_material),
        SortField::Modifier => symbols(folded, left.modifier, right.modifier),
        SortField::Coastal => left.coastal.cmp(&right.coastal),
        SortField::RiverPresence => left.river.is_some().cmp(&right.river.is_some()),
        SortField::RiverLevel => optional_values(
            left.river.as_ref().map(|value| value.level.0),
            right.river.as_ref().map(|value| value.level.0),
        ),
        SortField::HarborSuitability => {
            optional_floats(left.harbor_suitability, right.harbor_suitability)
        }
        SortField::MovementPresence => left
            .movement_assistance
            .is_some()
            .cmp(&right.movement_assistance.is_some()),
        SortField::MovementX => {
            optional_floats(movement_component(left, 0), movement_component(right, 0))
        }
        SortField::MovementY => {
            optional_floats(movement_component(left, 1), movement_component(right, 1))
        }
    }
}

fn symbols(folded: &[String], left: Option<SymbolId>, right: Option<SymbolId>) -> Ordering {
    optional_values(
        left.and_then(|value| symbol_key(folded, value)),
        right.and_then(|value| symbol_key(folded, value)),
    )
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

fn optional_floats(left: Option<f32>, right: Option<f32>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.partial_cmp(&right).unwrap_or(Ordering::Equal),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn movement_component(record: &LocationRecord, index: usize) -> Option<f32> {
    record
        .movement_assistance
        .and_then(|value| value.get(index).copied())
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
        SortField::Religion => (left.religion.is_none(), right.religion.is_none()),
        SortField::Culture => (left.culture.is_none(), right.culture.is_none()),
        SortField::RawMaterial => (left.raw_material.is_none(), right.raw_material.is_none()),
        SortField::Modifier => (left.modifier.is_none(), right.modifier.is_none()),
        SortField::RiverLevel => (left.river.is_none(), right.river.is_none()),
        SortField::HarborSuitability => (
            left.harbor_suitability.is_none(),
            right.harbor_suitability.is_none(),
        ),
        SortField::MovementX | SortField::MovementY => (
            left.movement_assistance.is_none(),
            right.movement_assistance.is_none(),
        ),
        _ => (false, false),
    };
    if nullable.0 != nullable.1 {
        ordering
    } else {
        ordering.reverse()
    }
}
