//! Facet selection, clearing and active filter counts against `FilterSet`.

use super::{
    FacetId, OptionValue, active_filter_count, clear, is_selected, selected_count, set_selected,
};
use crate::filter::{FilterSet, FloatRange};
use crate::model::{LocationKind, SymbolId};

type Probe = fn(&FilterSet) -> bool;

const SYMBOL: SymbolId = SymbolId(7);

/// One representative value per facet with a probe reading the `FilterSet` field it must change.
fn table() -> [(FacetId, OptionValue, Probe); FacetId::COUNT] {
    let some = OptionValue::Symbol(Some(SYMBOL));
    [
        (FacetId::Kind, OptionValue::Kind(LocationKind::Sea), |f| {
            f.kinds.contains(&LocationKind::Sea)
        }),
        (FacetId::Continent, some, |f| {
            f.continents.contains(&Some(SYMBOL))
        }),
        (FacetId::Subcontinent, some, |f| {
            f.subcontinents.contains(&Some(SYMBOL))
        }),
        (FacetId::Region, some, |f| f.regions.contains(&Some(SYMBOL))),
        (FacetId::Area, some, |f| f.areas.contains(&Some(SYMBOL))),
        (FacetId::Province, some, |f| {
            f.provinces.contains(&Some(SYMBOL))
        }),
        (FacetId::Topography, some, |f| {
            f.topographies.contains(&SYMBOL)
        }),
        (FacetId::Vegetation, some, |f| {
            f.vegetation.contains(&Some(SYMBOL))
        }),
        (FacetId::Climate, some, |f| {
            f.climates.contains(&Some(SYMBOL))
        }),
        (FacetId::Religion, some, |f| {
            f.religions.contains(&Some(SYMBOL))
        }),
        (FacetId::Culture, some, |f| {
            f.cultures.contains(&Some(SYMBOL))
        }),
        (FacetId::RawMaterial, some, |f| {
            f.raw_materials.contains(&Some(SYMBOL))
        }),
        (FacetId::Modifier, some, |f| {
            f.modifiers.contains(&Some(SYMBOL))
        }),
        (FacetId::Coastal, OptionValue::Flag(false), |f| {
            f.coastal.contains(&false)
        }),
        (FacetId::RiverPresence, OptionValue::Flag(true), |f| {
            f.river_presence.contains(&true)
        }),
        (FacetId::RiverLevelMin, OptionValue::Tier(2), |f| {
            f.river_level_min.contains(&2)
        }),
        (FacetId::RiverLevelMax, OptionValue::Tier(4), |f| {
            f.river_level_max.contains(&4)
        }),
        (FacetId::HarborPresence, OptionValue::Flag(true), |f| {
            f.harbor_presence.contains(&true)
        }),
        (FacetId::MovementPresence, OptionValue::Flag(false), |f| {
            f.movement_presence.contains(&false)
        }),
    ]
}

/// Sum of every facet's selection size.
fn total(filters: &FilterSet) -> usize {
    FacetId::ALL
        .into_iter()
        .map(|facet| selected_count(filters, facet))
        .sum()
}

#[test]
fn every_facet_toggles_its_own_field() {
    for (facet, value, probe) in table() {
        let mut filters = FilterSet::default();
        assert!(
            !probe(&filters) && !is_selected(&filters, facet, value),
            "{facet:?}"
        );
        assert!(set_selected(&mut filters, facet, value, true), "{facet:?}");
        assert!(
            probe(&filters) && is_selected(&filters, facet, value),
            "{facet:?}"
        );
        assert_eq!(selected_count(&filters, facet), 1, "{facet:?}");
        assert_eq!(total(&filters), 1, "{facet:?} leaked into another facet");
        assert!(
            !set_selected(&mut filters, facet, value, true),
            "repeat {facet:?}"
        );
        assert!(set_selected(&mut filters, facet, value, false), "{facet:?}");
        assert!(!probe(&filters) && total(&filters) == 0, "{facet:?}");
        assert!(
            !set_selected(&mut filters, facet, value, false),
            "repeat {facet:?}"
        );
        assert!(set_selected(&mut filters, facet, value, true));
        clear(&mut filters, facet);
        assert!(!probe(&filters) && total(&filters) == 0, "clear {facet:?}");
    }
}

#[test]
fn the_missing_value_is_selectable_where_the_facet_has_one() {
    for facet in FacetId::ALL {
        let mut filters = FilterSet::default();
        let changed = set_selected(&mut filters, facet, OptionValue::Symbol(None), true);
        let accepts = !matches!(
            facet,
            FacetId::Kind
                | FacetId::Topography
                | FacetId::Coastal
                | FacetId::RiverPresence
                | FacetId::RiverLevelMin
                | FacetId::RiverLevelMax
                | FacetId::HarborPresence
                | FacetId::MovementPresence
        );
        assert_eq!(changed, accepts, "{facet:?}");
        assert_eq!(
            is_selected(&filters, facet, OptionValue::Symbol(None)),
            accepts
        );
    }
}

#[test]
fn values_of_the_wrong_type_change_nothing() {
    let samples = [
        OptionValue::Kind(LocationKind::Land),
        OptionValue::Symbol(Some(SYMBOL)),
        OptionValue::Flag(true),
        OptionValue::Tier(3),
    ];
    for (facet, right, _) in table() {
        for sample in samples {
            let same_type = std::mem::discriminant(&sample) == std::mem::discriminant(&right);
            let mut filters = FilterSet::default();
            let changed = set_selected(&mut filters, facet, sample, true);
            assert_eq!(changed, same_type, "{facet:?} {sample:?}");
            assert_eq!(is_selected(&filters, facet, sample), same_type);
            assert_eq!(total(&filters), usize::from(same_type));
        }
    }
}

#[test]
fn selections_in_one_facet_do_not_touch_siblings() {
    let mut filters = FilterSet::default();
    let value = OptionValue::Symbol(Some(SYMBOL));
    assert!(set_selected(&mut filters, FacetId::Region, value, true));
    assert!(!is_selected(&filters, FacetId::Area, value));
    assert!(!is_selected(&filters, FacetId::Topography, value));
    assert!(set_selected(
        &mut filters,
        FacetId::RiverLevelMin,
        OptionValue::Tier(1),
        true
    ));
    assert!(!is_selected(
        &filters,
        FacetId::RiverLevelMax,
        OptionValue::Tier(1)
    ));
}

#[test]
fn active_count_follows_the_badge_rules() {
    assert_eq!(active_filter_count(&FilterSet::land_only()), 0);
    assert_eq!(
        active_filter_count(&FilterSet::default()),
        1,
        "no kinds differs from land only"
    );
    let mut filters = FilterSet::land_only();
    assert!(set_selected(
        &mut filters,
        FacetId::Kind,
        OptionValue::Kind(LocationKind::Sea),
        true
    ));
    assert_eq!(
        active_filter_count(&filters),
        1,
        "land and sea differs from land only"
    );
    // Two options of one facet count once.
    for tier in [1, 2] {
        assert!(set_selected(
            &mut filters,
            FacetId::RiverLevelMax,
            OptionValue::Tier(tier),
            true
        ));
    }
    assert_eq!(active_filter_count(&filters), 2);
    filters.search = "ras".to_owned();
    assert_eq!(active_filter_count(&filters), 3);
    filters.search = "   ".to_owned();
    assert_eq!(active_filter_count(&filters), 2);
    let mut filters = FilterSet::land_only();
    filters.rgb = Some(crate::model::MapColor(0x0001_0203));
    filters.show_impassable = false;
    filters.food_producing_only = true;
    assert_eq!(active_filter_count(&filters), 3);
    filters.harbor_range = FloatRange {
        min: Some(0.1),
        max: None,
    };
    filters.movement_x = FloatRange {
        min: None,
        max: Some(1.0),
    };
    assert_eq!(active_filter_count(&filters), 5);
    filters.movement_y = FloatRange {
        min: Some(-1.0),
        max: Some(1.0),
    };
    assert_eq!(active_filter_count(&filters), 6);
}
