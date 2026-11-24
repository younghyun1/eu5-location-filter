//! Facet selection, clearing and active filter counts against `FilterSet`.

use super::options::test_data::{dataset, english_options};
use super::{
    Bound, FacetId, FacetRows, FacetUi, OptionValue, RailSection, RangeField, SortState,
    active_filter_count, clear, column_for_sort, is_selected, selected_count, set_selected,
    sort_from_token, sort_token,
};
use crate::app::widget::ChecklistModel;
use crate::filter::{FilterSet, FloatRange, SortField};
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
    filters.rgb = Some(crate::model::MapColor(0x0102_03));
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

#[test]
fn facet_metadata_is_consistent() {
    for (position, facet) in FacetId::ALL.into_iter().enumerate() {
        assert_eq!(facet.index(), position);
        assert_eq!(facet.starts_folded(), facet != FacetId::Kind);
    }
    let mut keys: Vec<&str> = FacetId::ALL.iter().map(|facet| facet.key()).collect();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(keys.len(), FacetId::COUNT);
    let missing: Vec<FacetId> = FacetId::ALL
        .into_iter()
        .filter(|f| f.has_missing_option())
        .collect();
    assert_eq!(missing.len(), 6);
    assert_eq!(FacetId::Kind.section(), RailSection::Location);
    assert_eq!(FacetId::Province.section(), RailSection::Geography);
    assert_eq!(FacetId::Climate.section(), RailSection::Terrain);
    assert_eq!(FacetId::Culture.section(), RailSection::Society);
    assert_eq!(FacetId::Modifier.section(), RailSection::Economy);
    assert_eq!(FacetId::MovementPresence.section(), RailSection::MapData);
    // Sections never go backwards along the rail order.
    let order = |section: RailSection| RailSection::ALL.iter().position(|s| *s == section);
    assert!(
        FacetId::ALL
            .windows(2)
            .all(|pair| order(pair[0].section()) <= order(pair[1].section()))
    );
}

#[test]
fn range_rows_read_and_write_their_filter_field() {
    let mut filters = FilterSet::default();
    let range = FloatRange {
        min: Some(1.0),
        max: Some(2.0),
    };
    for field in RangeField::ALL {
        assert_eq!(field.get(&filters), FloatRange::default());
        *field.get_mut(&mut filters) = range;
        assert_eq!(field.get(&filters), range);
    }
    assert_eq!(filters.harbor_range, range);
    assert_eq!(filters.movement_x, range);
    assert_eq!(filters.movement_y, range);
    assert_eq!((Bound::Min.index(), Bound::Max.index()), (0, 1));
    assert_eq!(RangeField::MovementY.index(), 2);
}

#[test]
fn facet_rows_show_visible_options_with_checked_state() {
    let data = dataset();
    let options = english_options(&data);
    let list = options.get(FacetId::Vegetation);
    let mut ui = FacetUi::new(FacetId::Vegetation);
    assert!(ui.folded && ui.query.is_empty() && ui.visible.is_empty());
    ui.refresh(list);
    assert_eq!(ui.visible.len(), list.len());
    let mut filters = FilterSet::default();
    let forest = list.position(OptionValue::Symbol(Some(SymbolId(9))));
    assert!(forest.is_some());
    if let Some(index) = forest {
        assert!(set_selected(
            &mut filters,
            FacetId::Vegetation,
            OptionValue::Symbol(Some(SymbolId(9))),
            true
        ));
        let rows = FacetRows {
            facet: FacetId::Vegetation,
            options: list,
            visible: &ui.visible,
            filters: &filters,
        };
        assert_eq!(rows.row_count(), list.len());
        let checked: Vec<usize> = (0..rows.row_count())
            .filter(|row| rows.is_checked(*row))
            .collect();
        assert_eq!(checked.len(), 1);
        assert_eq!(checked.first().map(|row| rows.option(*row)), Some(index));
        assert_eq!(
            rows.label(checked.first().copied().unwrap_or(usize::MAX)),
            list.label(index)
        );
    }
    ui.query = "forest".to_owned();
    ui.refresh(list);
    assert_eq!(ui.visible.len(), 1);
    let rows = FacetRows {
        facet: FacetId::Vegetation,
        options: list,
        visible: &ui.visible,
        filters: &filters,
    };
    assert!(rows.is_checked(0));
    assert_eq!(rows.label(5), "");
    assert!(!rows.is_checked(5));
    assert_eq!(rows.option(5), u32::MAX);
}

#[test]
fn sort_tokens_round_trip_and_toggle() {
    for field in SortField::ALL {
        assert_eq!(sort_from_token(sort_token(field)), Some(field), "{field:?}");
        assert_eq!(column_for_sort(field).sort_field(), field, "{field:?}");
    }
    assert_eq!(sort_from_token("rgb"), Some(SortField::Color));
    assert_eq!(sort_from_token("bogus"), None);
    assert_eq!(sort_from_token(""), None);
    let state = SortState::default();
    assert_eq!((state.field, state.ascending), (SortField::Name, true));
    let reversed = state.toggled(SortField::Name);
    assert!(!reversed.ascending && reversed.field == SortField::Name);
    let other = reversed.toggled(SortField::Kind);
    assert!(other.ascending && other.field == SortField::Kind);
}
