//! Differential checks against the original scan, including both output strategies.

use super::{FilterEngine, FilterSet, FloatRange, SortField, bitmap::Bitmap, posting::Posting};
use crate::{
    AppError,
    model::{Dataset, LocationId, LocationKind, MapColor, RiverData, RiverLevel, SymbolId},
};
use std::{collections::HashSet, sync::Arc};

pub(super) fn varied_fixture() -> Dataset {
    let mut dataset = super::test_fixture::fixture();
    dataset
        .stored
        .dictionary
        .extend(["wheat", "iron", "東京 aaaa", "Ra's al-'Ain", "Kouřim"].map(str::to_owned));
    let Some(template) = dataset.stored.locations.first().cloned() else {
        return dataset;
    };
    dataset.stored.locations.clear();
    dataset.by_color.clear();
    for index in 0..257_u32 {
        let mut record = template.clone();
        record.id = LocationId(index);
        record.color = MapColor(index + 1);
        record.name = SymbolId(7 + index % 3);
        record.key = SymbolId(index % 2);
        record.kind = match index % 5 {
            0 => LocationKind::Land,
            1 => LocationKind::Sea,
            2 => LocationKind::Lake,
            3 => LocationKind::Impassable,
            _ => LocationKind::Unknown,
        };
        let optional = (index % 3 != 0).then_some(SymbolId(index % 7));
        record.topography = SymbolId(index % 7);
        record.vegetation = optional;
        record.climate = optional;
        record.religion = optional;
        record.culture = optional;
        record.raw_material = optional;
        record.modifier = optional;
        record.coastal = index % 2 == 0;
        record.hierarchy.continent = SymbolId(index % 4);
        record.hierarchy.subcontinent = SymbolId(index % 5);
        record.hierarchy.region = SymbolId(index % 6);
        record.hierarchy.area = SymbolId(index % 7);
        record.hierarchy.province = SymbolId(index % 8);
        record.harbor_suitability = (index % 3 != 0).then_some((index % 9) as f32 - 4.0);
        record.movement_assistance =
            (index % 4 != 0).then_some([(index % 11) as f32 - 5.0, (index % 13) as f32]);
        record.river = (index % 3 != 0).then_some(RiverData {
            level: RiverLevel((index % 5 + 1) as u8),
            has_source: false,
            has_confluence: false,
        });
        dataset.by_color.insert(record.color, record.id);
        dataset.stored.locations.push(record);
    }
    dataset
}

fn equal_to_scan(engine: &FilterEngine, filters: &FilterSet) {
    for field in SortField::ALL {
        for ascending in [true, false] {
            assert_eq!(
                engine.apply(filters, field, ascending),
                engine.apply_scan(filters, field, ascending),
                "{field:?}, ascending={ascending}, {filters:?}"
            );
        }
    }
}

#[test]
fn all_facets_and_ranges_match_scan_in_both_directions() -> Result<(), AppError> {
    let dataset = Arc::new(varied_fixture());
    let index = FilterEngine::build_stored_index(&dataset);
    let engine = FilterEngine::from_stored_index(dataset, index)?;
    equal_to_scan(&engine, &FilterSet::default());
    equal_to_scan(&engine, &FilterSet::land_only());
    for slot in 0..26 {
        let mut filters = FilterSet::default();
        let choices = HashSet::from([None, Some(SymbolId(1)), Some(SymbolId(5))]);
        let range = FloatRange {
            min: Some(-2.0),
            max: Some(3.0),
        };
        match slot {
            0 => filters
                .kinds
                .extend([LocationKind::Land, LocationKind::Unknown]),
            1 => filters.topographies.extend([SymbolId(2), SymbolId(4)]),
            2 => filters.vegetation = choices,
            3 => filters.climates = choices,
            4 => filters.continents = choices,
            5 => filters.subcontinents = choices,
            6 => filters.regions = choices,
            7 => filters.areas = choices,
            8 => filters.provinces = choices,
            9 => filters.religions = choices,
            10 => filters.cultures = choices,
            11 => filters.raw_materials = choices,
            12 => filters.modifiers = choices,
            13 => {
                filters.coastal.insert(true);
            }
            14 => {
                filters.river_presence.insert(false);
            }
            15 => {
                filters.harbor_presence.insert(false);
            }
            16 => {
                filters.movement_presence.insert(false);
            }
            17 => filters.food_producing_only = true,
            18 => filters.show_impassable = false,
            19 => filters.harbor_range = range,
            20 => filters.movement_x = range,
            21 => filters.movement_y = range,
            22 => filters.river_level_min.extend([2, 4]),
            23 => filters.river_level_max.extend([1, 3]),
            24 => filters.rgb = Some(MapColor(127)),
            _ => filters.rgb = Some(MapColor(999)),
        }
        equal_to_scan(&engine, &filters);
        filters.coastal.extend([true]);
        filters.search = "ras".to_owned();
        equal_to_scan(&engine, &filters);
    }
    Ok(())
}

#[test]
fn search_candidates_preserve_unicode_repetitions_and_short_queries() {
    let engine = FilterEngine::new(Arc::new(varied_fixture()));
    for query in [
        "",
        "a",
        "aa",
        "aaa",
        "aaaaa",
        "東京",
        "京",
        "kourim",
        "KOUŘIM",
        "Ra's al Ain",
        "ire",
        "zzzz",
        "rasain",
        "rimé",
        "\0",
        "a\0e",
    ] {
        equal_to_scan(
            &engine,
            &FilterSet {
                search: query.to_owned(),
                ..FilterSet::default()
            },
        );
    }
}

#[test]
fn range_edges_missing_and_contradictory_bounds_match_scan() {
    let engine = FilterEngine::new(Arc::new(varied_fixture()));
    for range in [
        FloatRange {
            min: Some(0.0),
            max: Some(0.0),
        },
        FloatRange {
            min: Some(5.0),
            max: Some(-5.0),
        },
        FloatRange {
            min: Some(f32::NAN),
            max: None,
        },
        FloatRange {
            min: Some(f32::NEG_INFINITY),
            max: Some(f32::INFINITY),
        },
    ] {
        for present in [
            HashSet::new(),
            HashSet::from([false]),
            HashSet::from([true, false]),
        ] {
            equal_to_scan(
                &engine,
                &FilterSet {
                    harbor_range: range,
                    harbor_presence: present.clone(),
                    ..FilterSet::default()
                },
            );
            equal_to_scan(
                &engine,
                &FilterSet {
                    movement_x: range,
                    movement_y: range,
                    movement_presence: present,
                    ..FilterSet::default()
                },
            );
        }
    }
}

#[test]
fn bitmap_padding_and_sparse_dense_postings_are_equivalent() {
    for count in [0, 1, 63, 64, 65, 127, 128, 129] {
        let full = Bitmap::full(count);
        assert_eq!(full.count(), count);
        assert_eq!(full.ids().count(), count);
        let ids: Vec<_> = full.ids().collect();
        let posting = Posting::new(ids.clone(), count);
        assert!(posting.validate(count).is_ok());
        let mut mask = Bitmap::empty(count);
        posting.union_into(&mut mask);
        assert_eq!(mask.ids().collect::<Vec<_>>(), ids);
    }
    assert!(super::sort::use_ranks(2, 257));
    assert!(!super::sort::use_ranks(200, 257));
}

#[test]
fn rejects_corrupt_ranks_postings_and_numeric_indexes() {
    let dataset = varied_fixture();
    let stored = FilterEngine::build_stored_index(&dataset);
    let mut corrupt = stored.clone();
    if let Some(order) = corrupt.orders.first_mut() {
        order.ascending_ranks.reverse();
    }
    assert!(corrupt.validate(&dataset).is_err());
    let mut corrupt = stored.clone();
    if let Some(facet) = corrupt.query.facets.first_mut() {
        facet.members = Posting::Sparse(vec![LocationId(999)]);
    }
    assert!(corrupt.validate(&dataset).is_err());
    let mut corrupt = stored.clone();
    corrupt.query.facets.pop();
    assert!(corrupt.validate(&dataset).is_err());
    let mut corrupt = stored.clone();
    if let Some(numeric) = corrupt.query.numeric.first_mut() {
        numeric.entries.reverse();
    }
    assert!(corrupt.validate(&dataset).is_err());
    let mut corrupt = stored.clone();
    if let Some(trigram) = corrupt.query.trigrams.first_mut() {
        trigram.ids.push(LocationId(999));
    }
    assert!(corrupt.validate(&dataset).is_err());
    assert_eq!(stored, FilterEngine::build_stored_index(&dataset));
}

#[test]
fn empty_dataset_has_valid_empty_indexes() {
    let mut dataset = varied_fixture();
    dataset.stored.locations.clear();
    let stored = FilterEngine::build_stored_index(&dataset);
    assert!(stored.validate(&dataset).is_ok());
    let engine = FilterEngine::new(Arc::new(dataset));
    assert!(
        engine
            .apply(&FilterSet::default(), SortField::Name, true)
            .is_empty()
    );
}
