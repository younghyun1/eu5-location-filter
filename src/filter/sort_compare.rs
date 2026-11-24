//! Offline column comparisons with nulls last and deterministic ID tie-breaking.

use super::SortField;
use super::text::fold_sort;
use crate::lang_pack::ResolvedText;
use crate::model::{Dataset, LocationId, LocationRecord, SymbolId};
use std::cmp::Ordering;

/// Folded sort keys, computed once per order build so comparisons only read slices.
///
/// Label-ordered columns need `names` and `labels`; the identifier column needs
/// `identifiers`. A build fills only the vectors its columns read, and a missing key sorts as
/// null, so a key set built for one group of columns cannot silently order another.
#[derive(Debug, Default)]
pub(super) struct SortKeys {
    /// `fold_sort` of the display name, per location ID.
    pub(super) names: Vec<String>,
    /// `fold_sort` of the display label, per dataset symbol.
    pub(super) labels: Vec<String>,
    /// `fold_sort` of the raw dictionary entry, per dataset symbol.
    pub(super) identifiers: Vec<String>,
}

impl SortKeys {
    /// Keys for the language-independent columns, which read only internal identifiers.
    pub(super) fn identifiers(dataset: &Dataset) -> Self {
        Self {
            identifiers: dataset
                .stored
                .dictionary
                .iter()
                .map(|value| fold_sort(value))
                .collect(),
            ..Self::default()
        }
    }

    /// Keys for the label-ordered columns of one language.
    pub(super) fn labels(text: &ResolvedText<'_>) -> Self {
        Self {
            names: text.names.iter().map(|value| fold_sort(value)).collect(),
            labels: text.labels.iter().map(|value| fold_sort(value)).collect(),
            identifiers: Vec::new(),
        }
    }
}

pub(super) fn compare(
    dataset: &Dataset,
    keys: &SortKeys,
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
    let ordering = compare_records(keys, left_record, right_record, field);
    let directed = if ascending {
        ordering
    } else {
        reverse_non_null(ordering, field, left_record, right_record)
    };
    directed.then_with(|| left.cmp(&right))
}

fn compare_records(
    keys: &SortKeys,
    left: &LocationRecord,
    right: &LocationRecord,
    field: SortField,
) -> Ordering {
    let labels = keys.labels.as_slice();
    match field {
        SortField::Color => left.color.cmp(&right.color),
        SortField::Name => optional_values(
            key(&keys.names, left.id.0),
            key(&keys.names, right.id.0),
        ),
        SortField::Identifier => symbols(&keys.identifiers, Some(left.key), Some(right.key)),
        SortField::Kind => left.kind.cmp(&right.kind),
        SortField::Topography => symbols(labels, Some(left.topography), Some(right.topography)),
        SortField::Vegetation => symbols(labels, left.vegetation, right.vegetation),
        SortField::Climate => symbols(labels, left.climate, right.climate),
        SortField::Continent => symbols(
            labels,
            Some(left.hierarchy.continent),
            Some(right.hierarchy.continent),
        ),
        SortField::Subcontinent => symbols(
            labels,
            Some(left.hierarchy.subcontinent),
            Some(right.hierarchy.subcontinent),
        ),
        SortField::Region => symbols(
            labels,
            Some(left.hierarchy.region),
            Some(right.hierarchy.region),
        ),
        SortField::Area => symbols(labels, Some(left.hierarchy.area), Some(right.hierarchy.area)),
        SortField::Province => symbols(
            labels,
            Some(left.hierarchy.province),
            Some(right.hierarchy.province),
        ),
        SortField::Religion => symbols(labels, left.religion, right.religion),
        SortField::Culture => symbols(labels, left.culture, right.culture),
        SortField::RawMaterial => symbols(labels, left.raw_material, right.raw_material),
        SortField::Modifier => symbols(labels, left.modifier, right.modifier),
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
        SortField::StaticPopulationCapacity => optional_values(
            left.static_population_capacity.map(|value| value.total.0),
            right.static_population_capacity.map(|value| value.total.0),
        ),
        SortField::EquatorCapacity => optional_values(
            left.static_population_capacity.map(|value| value.equator.0),
            right
                .static_population_capacity
                .map(|value| value.equator.0),
        ),
    }
}

fn symbols(folded: &[String], left: Option<SymbolId>, right: Option<SymbolId>) -> Ordering {
    optional_values(
        left.and_then(|value| key(folded, value.0)),
        right.and_then(|value| key(folded, value.0)),
    )
}

fn key(folded: &[String], index: u32) -> Option<&str> {
    usize::try_from(index)
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
        SortField::StaticPopulationCapacity | SortField::EquatorCapacity => (
            left.static_population_capacity.is_none(),
            right.static_population_capacity.is_none(),
        ),
        _ => (false, false),
    };
    if nullable.0 != nullable.1 {
        ordering
    } else {
        ordering.reverse()
    }
}
