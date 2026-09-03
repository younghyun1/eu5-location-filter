//! Build-time embedded location and filter-index bundles.

use crate::filter::StoredFilterIndex;
use crate::model::Dataset;
use crate::{AppError, index_storage, storage};

const LOCATIONS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/eu5-locations.bitcode.zst"));
const INDEXES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/eu5-indexes.bitcode.zst"));

/// Decodes both committed bundles into memory without accessing game files.
pub fn load() -> Result<(Dataset, StoredFilterIndex), AppError> {
    if LOCATIONS.is_empty() || INDEXES.is_empty() {
        return Err(AppError::InvalidData(
            "embedded EU5 bundles are missing from this build".to_owned(),
        ));
    }
    Ok((
        storage::decode_blob(LOCATIONS)?,
        index_storage::decode_index(INDEXES)?,
    ))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::sync::Arc;

    use super::load;
    use crate::filter::{FilterEngine, FilterSet, SortField};

    #[test]
    fn committed_bundles_decode_and_pair() {
        let loaded = load();
        assert!(loaded.is_ok());
        let Ok((dataset, index)) = loaded else {
            return;
        };
        assert_eq!(dataset.stored.locations.len(), 28_573);
        assert_eq!(
            dataset
                .stored
                .locations
                .iter()
                .filter(|record| record.kind == crate::model::LocationKind::Impassable)
                .count(),
            1_902
        );
        assert!(dataset.stored.locations.iter().all(|record| {
            dataset.symbol(record.topography) != Some("salt_pans")
                || (record.kind == crate::model::LocationKind::Impassable
                    && record.static_population_capacity.is_none())
        }));
        let heard_island = dataset
            .stored
            .locations
            .iter()
            .find(|record| dataset.symbol(record.key) == Some("heard_island"));
        assert!(heard_island.is_some_and(|record| {
            record.kind == crate::model::LocationKind::Impassable
                && record.static_population_capacity.is_none()
        }));
        let tortosa = dataset
            .stored
            .locations
            .iter()
            .find(|record| dataset.symbol(record.key) == Some("tortosa"));
        assert_eq!(
            tortosa.and_then(|record| record.river.map(|river| river.level.0)),
            Some(1)
        );
        let kotor = dataset
            .stored
            .locations
            .iter()
            .find(|record| dataset.symbol(record.key) == Some("kotor"));
        assert_eq!(
            kotor.and_then(|record| {
                record
                    .static_population_capacity
                    .map(|capacity| capacity.equator.0)
            }),
            Some(2_661)
        );
        let dataset = Arc::new(dataset);
        let engine = FilterEngine::from_stored_index(Arc::clone(&dataset), index);
        assert!(engine.is_ok());
        let Ok(engine) = engine else { return };
        assert_eq!(
            engine
                .apply(&FilterSet::default(), SortField::Name, true)
                .len(),
            28_573
        );
        let food_results = engine.apply(
            &FilterSet {
                food_producing_only: true,
                ..FilterSet::default()
            },
            SortField::Name,
            true,
        );
        assert_eq!(food_results.len(), 12_035);
        assert!(food_results.iter().all(|id| {
            dataset
                .location(*id)
                .and_then(|record| record.raw_material)
                .and_then(|material| dataset.symbol(material))
                .is_some_and(crate::model::is_food_producing)
        }));
        let represented_food_materials: BTreeSet<&str> = food_results
            .iter()
            .filter_map(|id| dataset.location(*id))
            .filter_map(|record| record.raw_material)
            .filter_map(|material| dataset.symbol(material))
            .collect();
        assert_eq!(
            represented_food_materials,
            BTreeSet::from([
                "beeswax",
                "fish",
                "fruit",
                "fur",
                "legumes",
                "livestock",
                "maize",
                "millet",
                "olives",
                "potato",
                "rice",
                "wheat",
                "wild_game",
                "wool",
            ])
        );
        for search in ["N'Goussa", "N Goussa", "ngoussa", "n_goussa"] {
            let results = engine.apply(
                &FilterSet {
                    search: search.to_owned(),
                    ..FilterSet::default()
                },
                SortField::Name,
                true,
            );
            assert_eq!(results.len(), 1, "unexpected result count for {search:?}");
            assert_eq!(
                results
                    .first()
                    .and_then(|id| dataset.location(*id))
                    .and_then(|record| dataset.symbol(record.key)),
                Some("ngoussa")
            );
        }
        let mut heard_filter = FilterSet {
            search: "heard_island".to_owned(),
            show_impassable: false,
            ..FilterSet::default()
        };
        assert!(
            engine
                .apply(&heard_filter, SortField::Name, true)
                .is_empty()
        );
        heard_filter.show_impassable = true;
        assert_eq!(engine.apply(&heard_filter, SortField::Name, true).len(), 1);
    }
}
