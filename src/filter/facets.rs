//! Offline categorical postings, including explicit missing and boolean values.

use bitcode::{Decode, Encode};
use std::collections::BTreeMap;

use super::posting::Posting;
use crate::{
    AppError,
    model::{Dataset, LocationKind, LocationRecord, SymbolId, is_food_producing},
};

#[derive(Clone, Copy, Debug, Decode, Encode, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum FacetKey {
    Kind(LocationKind),
    Topography(SymbolId),
    Vegetation(Option<SymbolId>),
    Climate(Option<SymbolId>),
    Continent(Option<SymbolId>),
    Subcontinent(Option<SymbolId>),
    Region(Option<SymbolId>),
    Area(Option<SymbolId>),
    Province(Option<SymbolId>),
    Religion(Option<SymbolId>),
    Culture(Option<SymbolId>),
    RawMaterial(Option<SymbolId>),
    Modifier(Option<SymbolId>),
    Coastal(bool),
    River(bool),
    Harbor(bool),
    Movement(bool),
    Food(bool),
}

#[derive(Clone, Debug, Decode, Encode, PartialEq)]
pub(super) struct StoredFacet {
    pub(super) key: FacetKey,
    pub(super) members: Posting,
}

const FACETS_PER_RECORD: usize = 18;

pub(super) fn build(dataset: &Dataset) -> Vec<StoredFacet> {
    let mut entries = BTreeMap::<_, Vec<_>>::new();
    for record in &dataset.stored.locations {
        for key in keys(dataset, record) {
            entries.entry(key).or_default().push(record.id);
        }
    }
    entries
        .into_iter()
        .map(|(key, ids)| StoredFacet {
            key,
            members: Posting::new(ids, dataset.stored.locations.len()),
        })
        .collect()
}

pub(super) fn lookup(facets: &[StoredFacet], key: FacetKey) -> Option<&Posting> {
    facets
        .binary_search_by_key(&key, |facet| facet.key)
        .ok()
        .and_then(|index| facets.get(index))
        .map(|facet| &facet.members)
}

pub(super) fn validate(facets: &[StoredFacet], dataset: &Dataset) -> Result<(), AppError> {
    let count = dataset.stored.locations.len();
    if facets.len() > count * FACETS_PER_RECORD
        || facets
            .windows(2)
            .any(|pair| !matches!(pair, [left, right] if left.key < right.key))
    {
        return Err(AppError::InvalidData(
            "facet keys are duplicated, unordered, or excessive".to_owned(),
        ));
    }
    // Checking membership and total cardinality proves complete field coverage without
    // constructing a second index. Each record has exactly one key per field.
    let mut total = 0;
    for facet in facets {
        facet.members.validate(count)?;
        let members = facet.members.count();
        if members == 0
            || !facet.members.all(|id| {
                dataset
                    .location(id)
                    .is_some_and(|record| matches_key(dataset, record, facet.key))
            })
        {
            return Err(AppError::InvalidData(
                "facet membership does not match the dataset".to_owned(),
            ));
        }
        total += members;
    }
    if total != count * FACETS_PER_RECORD {
        return Err(AppError::InvalidData(
            "facet index is incomplete".to_owned(),
        ));
    }
    Ok(())
}

fn matches_key(dataset: &Dataset, record: &LocationRecord, key: FacetKey) -> bool {
    use FacetKey as K;
    match key {
        K::Kind(value) => value == record.kind,
        K::Topography(value) => value == record.topography,
        K::Vegetation(value) => value == record.vegetation,
        K::Climate(value) => value == record.climate,
        K::Continent(value) => value == Some(record.hierarchy.continent),
        K::Subcontinent(value) => value == Some(record.hierarchy.subcontinent),
        K::Region(value) => value == Some(record.hierarchy.region),
        K::Area(value) => value == Some(record.hierarchy.area),
        K::Province(value) => value == Some(record.hierarchy.province),
        K::Religion(value) => value == record.religion,
        K::Culture(value) => value == record.culture,
        K::RawMaterial(value) => value == record.raw_material,
        K::Modifier(value) => value == record.modifier,
        K::Coastal(value) => value == record.coastal,
        K::River(value) => value == record.river.is_some(),
        K::Harbor(value) => value == record.harbor_suitability.is_some(),
        K::Movement(value) => value == record.movement_assistance.is_some(),
        K::Food(value) => {
            value
                == record
                    .raw_material
                    .and_then(|id| dataset.symbol(id))
                    .is_some_and(is_food_producing)
        }
    }
}

fn keys(dataset: &Dataset, record: &LocationRecord) -> [FacetKey; FACETS_PER_RECORD] {
    use FacetKey as K;
    [
        K::Kind(record.kind),
        K::Topography(record.topography),
        K::Vegetation(record.vegetation),
        K::Climate(record.climate),
        K::Continent(Some(record.hierarchy.continent)),
        K::Subcontinent(Some(record.hierarchy.subcontinent)),
        K::Region(Some(record.hierarchy.region)),
        K::Area(Some(record.hierarchy.area)),
        K::Province(Some(record.hierarchy.province)),
        K::Religion(record.religion),
        K::Culture(record.culture),
        K::RawMaterial(record.raw_material),
        K::Modifier(record.modifier),
        K::Coastal(record.coastal),
        K::River(record.river.is_some()),
        K::Harbor(record.harbor_suitability.is_some()),
        K::Movement(record.movement_assistance.is_some()),
        K::Food(
            record
                .raw_material
                .and_then(|id| dataset.symbol(id))
                .is_some_and(is_food_producing),
        ),
    ]
}
