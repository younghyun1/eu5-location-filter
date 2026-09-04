use std::collections::HashSet;
use std::sync::Arc;

use crate::model::{LocationId, LocationKind, SymbolId};

use super::test_fixture::fixture;
use super::{FilterEngine, FilterSet, FloatRange, SortField};

#[test]
fn filters_use_or_within_fields_and_and_across_fields() {
    let dataset = fixture();
    let engine = FilterEngine::new(Arc::new(dataset));
    let mut filters = FilterSet::default();
    filters
        .kinds
        .extend([LocationKind::Land, LocationKind::Lake]);
    filters.coastal.insert(true);
    let ids = engine.apply(&filters, SortField::Name, true);
    assert_eq!(ids, vec![LocationId(0)]);
}

#[test]
fn supports_missing_inclusive_ranges_and_unicode_search() {
    let dataset = fixture();
    let engine = FilterEngine::new(Arc::new(dataset));
    let mut filters = FilterSet {
        religions: HashSet::from([None]),
        harbor_presence: HashSet::from([true]),
        harbor_range: FloatRange {
            min: Some(0.5),
            max: Some(0.5),
        },
        search: "ÉIRE".to_owned(),
        ..FilterSet::default()
    };
    assert_eq!(
        engine.apply(&filters, SortField::Name, true),
        vec![LocationId(0)]
    );
    filters.harbor_presence = HashSet::from([false]);
    assert!(engine.apply(&filters, SortField::Name, true).is_empty());
}

#[test]
fn nullable_and_boolean_checklists_allow_multiple_selections() {
    let mut dataset = fixture();
    dataset.stored.locations[1].hierarchy.region = SymbolId(4);
    let engine = FilterEngine::new(Arc::new(dataset));
    let filters = FilterSet {
        religions: HashSet::from([None, Some(SymbolId(3))]),
        regions: HashSet::from([Some(SymbolId(0)), Some(SymbolId(4))]),
        coastal: HashSet::from([true, false]),
        harbor_presence: HashSet::from([true, false]),
        ..FilterSet::default()
    };
    assert_eq!(
        engine.apply(&filters, SortField::Name, true),
        vec![LocationId(0), LocationId(1)]
    );
}

#[test]
fn interactive_default_selects_only_land() {
    let engine = FilterEngine::new(Arc::new(fixture()));
    assert_eq!(
        engine.apply(&FilterSet::land_only(), SortField::Name, true),
        vec![LocationId(0)]
    );
}

#[test]
fn toggles_impassables_and_preserves_visible_selection() {
    let dataset = fixture();
    let engine = FilterEngine::new(Arc::new(dataset));
    let filters = FilterSet {
        show_impassable: false,
        ..FilterSet::default()
    };
    let ids = engine.apply(&filters, SortField::RiverLevel, false);
    assert_eq!(ids, vec![LocationId(0)]);
    assert_eq!(
        FilterEngine::preserve_selection(Some(LocationId(0)), &ids),
        Some(LocationId(0))
    );
    assert_eq!(
        FilterEngine::preserve_selection(Some(LocationId(1)), &ids),
        None
    );
}

#[test]
fn river_minimum_excludes_missing_but_maximum_includes_it() {
    let engine = FilterEngine::new(Arc::new(fixture()));
    let mut filters = FilterSet {
        river_level_max: HashSet::from([1]),
        ..FilterSet::default()
    };
    assert_eq!(
        engine.apply(&filters, SortField::Name, true),
        vec![LocationId(0), LocationId(1)]
    );

    filters = FilterSet {
        river_level_min: HashSet::from([0]),
        ..FilterSet::default()
    };
    assert_eq!(
        engine.apply(&filters, SortField::Name, true),
        vec![LocationId(0)]
    );
}

#[test]
fn food_producing_filter_can_be_enabled_and_disabled() {
    let mut dataset = fixture();
    dataset
        .stored
        .dictionary
        .extend(["wheat".to_owned(), "iron".to_owned()]);
    dataset.stored.locations[0].raw_material = Some(SymbolId(5));
    dataset.stored.locations[1].raw_material = Some(SymbolId(6));
    let engine = FilterEngine::new(Arc::new(dataset));
    let mut filters = FilterSet {
        food_producing_only: true,
        ..FilterSet::default()
    };
    assert_eq!(
        engine.apply(&filters, SortField::Name, true),
        vec![LocationId(0)]
    );

    filters.food_producing_only = false;
    assert_eq!(
        engine.apply(&filters, SortField::Name, true),
        vec![LocationId(0), LocationId(1)]
    );
}

#[test]
fn food_producing_filter_combines_with_exact_raw_material() {
    let mut dataset = fixture();
    dataset
        .stored
        .dictionary
        .extend(["wheat".to_owned(), "iron".to_owned()]);
    dataset.stored.locations[0].raw_material = Some(SymbolId(5));
    dataset.stored.locations[1].raw_material = Some(SymbolId(6));
    let engine = FilterEngine::new(Arc::new(dataset));
    let filters = FilterSet {
        food_producing_only: true,
        raw_materials: HashSet::from([Some(SymbolId(6))]),
        ..FilterSet::default()
    };
    assert!(engine.apply(&filters, SortField::Name, true).is_empty());
}

#[test]
fn ascii_search_matches_localized_diacritics() {
    let mut dataset = fixture();
    dataset.stored.dictionary[1] = "Kouřim".to_owned();
    let engine = FilterEngine::new(Arc::new(dataset));
    let filters = FilterSet {
        search: "kourim".to_owned(),
        ..FilterSet::default()
    };
    assert_eq!(
        engine.apply(&filters, SortField::Name, true),
        vec![LocationId(0)]
    );
}

#[test]
fn search_ignores_punctuation_and_whitespace_in_names_and_identifiers() {
    let mut dataset = fixture();
    dataset.stored.dictionary[0] = "ras_al_ain".to_owned();
    dataset.stored.dictionary[1] = "Ra's al-'Ain".to_owned();
    let engine = FilterEngine::new(Arc::new(dataset));
    for search in [
        "rasalain",
        "ras al ain",
        "ras_al_ain",
        "Ra's al-'Ain",
        "RAS—AL—ʿAIN",
        "ras\u{00a0}\t al\n ain",
    ] {
        let filters = FilterSet {
            search: search.to_owned(),
            ..FilterSet::default()
        };
        assert_eq!(
            engine.apply(&filters, SortField::Name, true),
            vec![LocationId(0)],
            "search did not match: {search:?}"
        );
    }
}

#[test]
fn search_does_not_cross_the_name_identifier_boundary() {
    let mut dataset = fixture();
    dataset.stored.dictionary[0] = "bar".to_owned();
    dataset.stored.dictionary[1] = "foo".to_owned();
    let engine = FilterEngine::new(Arc::new(dataset));
    let filters = FilterSet {
        search: "obar".to_owned(),
        ..FilterSet::default()
    };
    assert!(engine.apply(&filters, SortField::Name, true).is_empty());
}

#[test]
fn punctuation_insensitive_search_preserves_sort_collation() {
    let mut dataset = fixture();
    dataset.stored.dictionary[1] = "az".to_owned();
    dataset.stored.dictionary[4] = "a-z".to_owned();
    let engine = FilterEngine::new(Arc::new(dataset));
    assert_eq!(
        engine.apply(&FilterSet::default(), SortField::Name, true),
        vec![LocationId(1), LocationId(0)]
    );
}

#[test]
fn rejects_the_previous_search_index_format() {
    let dataset = Arc::new(fixture());
    let mut stored = FilterEngine::build_stored_index(&dataset);
    stored.format_version = 4;
    assert!(FilterEngine::from_stored_index(dataset, stored).is_err());
}

#[test]
fn every_column_has_precomputed_orders_with_nulls_last() {
    let engine = FilterEngine::new(Arc::new(fixture()));
    for field in SortField::ALL {
        assert_eq!(engine.apply(&FilterSet::default(), field, true).len(), 2);
        assert_eq!(engine.apply(&FilterSet::default(), field, false).len(), 2);
    }
    assert_eq!(
        engine.apply(&FilterSet::default(), SortField::HarborSuitability, false),
        vec![LocationId(0), LocationId(1)]
    );
}
