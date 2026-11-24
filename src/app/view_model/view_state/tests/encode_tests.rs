//! Canonical encoding: the specification examples, ordering, omission and the length cap.

use super::fixture::{
    BOREAL, CATHOLIC, EUROPE, EVORA, FIRST_LONG_KEY, FOREST, HILLS, LONG_KEY_COUNT, MOUNTAINS,
    STOCKHOLM, UPPLAND, WHEAT, dataset, encode,
};
use crate::app::view_model::SortState;
use crate::app::view_model::view_state::VIEW_STATE_MAX_LEN;
use crate::filter::{FilterSet, FloatRange, SortField};
use crate::model::{LocationKind, MapColor, SymbolId};

fn sorted(field: SortField, ascending: bool) -> SortState {
    SortState { field, ascending }
}

#[test]
fn default_state_is_the_bare_version() {
    let data = dataset();
    assert_eq!(
        encode(&data, &FilterSet::land_only(), SortState::default(), None),
        "s1"
    );
}

#[test]
fn specification_examples_encode() {
    let data = dataset();

    let mut second = FilterSet::land_only();
    "river".clone_into(&mut second.search);
    second.kinds.insert(LocationKind::Sea);
    assert_eq!(
        encode(
            &data,
            &second,
            sorted(SortField::HarborSuitability, false),
            None
        ),
        "s1~q.river~k.land*sea~s.harbor*d"
    );

    let mut third = FilterSet::land_only();
    "new york".clone_into(&mut third.search);
    third.topographies.extend([MOUNTAINS, HILLS]);
    third.climates.insert(None);
    third.show_impassable = false;
    assert_eq!(
        encode(&data, &third, SortState::default(), None),
        "s1~q.new!20york~tp.hills*mountains~cl.!~si.0"
    );

    let mut fourth = FilterSet::land_only();
    "서울".clone_into(&mut fourth.search);
    fourth.rgb = Some(MapColor(0x1A_2B3C));
    fourth.coastal.insert(true);
    fourth.river_level_min.extend([3, 2]);
    fourth.harbor_range = FloatRange {
        min: Some(0.5),
        max: None,
    };
    assert_eq!(
        encode(&data, &fourth, SortState::default(), None),
        "s1~q.!EC!84!9C!EC!9A!B8~c.1A2B3C~co.yes~lm.2*3~hr.0.5*"
    );

    let mut fifth = FilterSet::default();
    fifth.raw_materials.extend([Some(WHEAT), None]);
    fifth.food_producing_only = true;
    fifth.movement_x = FloatRange {
        min: Some(-1.5),
        max: Some(2.0),
    };
    assert_eq!(
        encode(
            &data,
            &fifth,
            sorted(SortField::Name, false),
            Some("stockholm")
        ),
        "s1~k.~rm.!*wheat~fo.1~mx.-1.5*2~s.name*d~l.stockholm"
    );
}

#[test]
fn every_field_in_table_order() {
    let data = dataset();
    let mut filters = FilterSet::land_only();
    "a".clone_into(&mut filters.search);
    filters.kinds.insert(LocationKind::Impassable);
    filters.topographies.insert(HILLS);
    filters.vegetation.insert(Some(FOREST));
    filters.climates.insert(Some(BOREAL));
    filters.continents.insert(Some(EUROPE));
    filters.subcontinents.insert(Some(UPPLAND));
    filters.regions.insert(Some(UPPLAND));
    filters.areas.insert(Some(UPPLAND));
    filters.provinces.insert(Some(UPPLAND));
    filters.religions.insert(Some(CATHOLIC));
    filters.cultures.insert(None);
    filters.raw_materials.insert(Some(WHEAT));
    filters.food_producing_only = true;
    filters.modifiers.insert(None);
    filters.rgb = Some(MapColor(0x00_00FF));
    filters.coastal.extend([true, false]);
    filters.river_presence.insert(true);
    filters.river_level_min.insert(1);
    filters.river_level_max.insert(5);
    filters.harbor_presence.insert(false);
    filters.harbor_range = FloatRange {
        min: None,
        max: Some(1.0),
    };
    filters.movement_presence.insert(true);
    filters.movement_x = FloatRange {
        min: Some(0.25),
        max: None,
    };
    filters.movement_y = FloatRange {
        min: Some(-3.0),
        max: Some(3.0),
    };
    filters.show_impassable = false;
    assert_eq!(
        encode(
            &data,
            &filters,
            sorted(SortField::Color, true),
            Some("évora")
        ),
        "s1~q.a~k.impassable*land~tp.hills~vg.forest~cl.boreal~ct.europe~sc.uppland~rg.uppland\
         ~ar.uppland~pv.uppland~rl.catholic~cu.!~rm.wheat~fo.1~md.!~c.0000FF~co.no*yes\
         ~rp.present~lm.1~lx.5~hp.missing~hr.*1~mp.present~mx.0.25*~my.-3*3~si.0~s.color*a\
         ~l.!C3!A9vora"
    );
}

#[test]
fn items_sort_by_encoded_bytes_and_missing_sorts_first() {
    let data = dataset();
    let mut filters = FilterSet::land_only();
    filters
        .raw_materials
        .extend([Some(WHEAT), None, Some(EVORA)]);
    filters.river_level_max.extend([5, 3, 4]);
    assert_eq!(
        encode(&data, &filters, SortState::default(), None),
        "s1~rm.!*!C3!A9vora*wheat~lx.3*4*5"
    );
}

#[test]
fn values_the_decoder_rejects_are_omitted() {
    let data = dataset();
    let mut filters = FilterSet::land_only();
    "tab\there".clone_into(&mut filters.search);
    filters.continents.insert(None);
    filters.provinces.insert(None);
    filters.topographies.insert(SymbolId(9_999));
    filters.river_level_min.insert(0);
    filters.harbor_range = FloatRange {
        min: Some(f32::NAN),
        max: Some(f32::INFINITY),
    };
    filters.movement_x = FloatRange {
        min: Some(-0.0),
        max: Some(f32::NEG_INFINITY),
    };
    filters.rgb = Some(MapColor(0x0100_0000));
    let long_key = "k".repeat(129);
    assert_eq!(
        encode(&data, &filters, SortState::default(), Some(&long_key)),
        "s1~mx.0*"
    );
    filters.search = "a".repeat(257);
    assert_eq!(
        encode(&data, &filters, SortState::default(), Some("")),
        "s1~mx.0*"
    );
}

#[test]
fn numbers_use_the_shortest_round_trip_form() {
    let data = dataset();
    let mut filters = FilterSet::land_only();
    filters.harbor_range = FloatRange {
        min: Some(0.1),
        max: Some(1e30),
    };
    filters.movement_y = FloatRange {
        min: Some(f32::MIN_POSITIVE),
        max: Some(-2.5),
    };
    let encoded = encode(&data, &filters, SortState::default(), None);
    assert!(
        encoded.starts_with("s1~hr.0.1*1000000000000000000000000000000~my."),
        "{encoded}"
    );
    assert!(encoded.ends_with("*-2.5"), "{encoded}");
    assert!(!encoded.contains('e'), "{encoded}");
}

#[test]
fn oversized_state_reports_the_default() {
    let data = dataset();
    let mut filters = FilterSet::land_only();
    filters
        .provinces
        .extend((0..LONG_KEY_COUNT).map(|index| Some(SymbolId(FIRST_LONG_KEY + index))));
    assert_eq!(encode(&data, &filters, SortState::default(), None), "s1");
    let mut fits = FilterSet::land_only();
    fits.provinces
        .extend((0..20).map(|index| Some(SymbolId(FIRST_LONG_KEY + index))));
    let encoded = encode(&data, &fits, SortState::default(), None);
    assert!(
        encoded.len() > 1_000 && encoded.len() <= VIEW_STATE_MAX_LEN,
        "{}",
        encoded.len()
    );
}

#[test]
fn hash_set_order_does_not_change_the_string() {
    let data = dataset();
    let mut first = FilterSet::land_only();
    let mut second = FilterSet::land_only();
    first.topographies.extend([HILLS, MOUNTAINS, STOCKHOLM]);
    second.topographies.extend([STOCKHOLM, MOUNTAINS, HILLS]);
    assert_eq!(
        encode(&data, &first, SortState::default(), None),
        encode(&data, &second, SortState::default(), None)
    );
}
