//! The detail model: field order, values, tones and missing values.

use crate::app::i18n::{Locale, Text};
use crate::app::view_model::options::test_data::{dataset, korean_pack};
use crate::app::view_model::{CellTone, DetailField, DetailGroup, LocationDetail, build_detail};
use crate::lang_pack::Labels;
use crate::model::{Dataset, LocationId};

fn detail_of(data: &Dataset, location: u32) -> Option<LocationDetail> {
    build_detail(Labels::new(data, None), Locale::EnUs, LocationId(location))
}

/// `(label, value, tone, mono)` of every field of a group.
fn rows(group: &DetailGroup) -> Vec<(Text, &str, CellTone, bool)> {
    group
        .fields
        .iter()
        .map(|field: &DetailField| (field.label, field.value.as_str(), field.tone, field.mono))
        .collect()
}

fn group(detail: &LocationDetail, index: usize) -> Option<&DetailGroup> {
    detail.groups.get(index)
}

#[test]
fn stockholm_header_and_breadcrumb() {
    let data = dataset();
    let detail = detail_of(&data, 0);
    assert!(detail.is_some());
    let Some(detail) = detail else { return };
    assert_eq!(detail.id, LocationId(0));
    assert_eq!(detail.hex, "#1A2B3C");
    assert_eq!(detail.rgb, "rgb(26, 43, 60)");
    assert_eq!(detail.name, "Stockholm");
    assert_eq!(detail.key, "stockholm");
    assert_eq!(detail.color.0, 0x1A_2B3C);
    assert_eq!(
        detail.breadcrumb,
        ["Europe", "Kouřim", "Ra's al-'Ain", "Москва", "東京"]
    );
    let titles: Vec<Text> = detail.groups.iter().map(|group| group.title).collect();
    assert_eq!(
        titles,
        [
            Text::DetailGroupClassification,
            Text::DetailGroupSociety,
            Text::DetailGroupEconomy,
            Text::DetailGroupCoast,
            Text::DetailGroupRiver,
            Text::DetailGroupCapacity,
        ]
    );
    assert!(detail.groups.iter().all(|group| (group.note.is_some()) == (group.title == Text::DetailGroupCapacity)));
}

#[test]
fn stockholm_fields_in_contract_order() {
    let data = dataset();
    let Some(detail) = detail_of(&data, 0) else {
        assert!(detail_of(&data, 0).is_some());
        return;
    };
    let ink = CellTone::Ink;
    let expect = |index: usize, expected: Vec<(Text, &str, CellTone, bool)>| {
        assert_eq!(group(&detail, index).map(rows), Some(expected), "group {index}");
    };
    expect(
        0,
        vec![
            (Text::DetailFieldType, "Land", ink, false),
            (Text::DetailFieldTopography, "hills", ink, false),
            (Text::DetailFieldVegetation, "forest", ink, false),
            (Text::DetailFieldClimate, "boreal", ink, false),
        ],
    );
    expect(
        1,
        vec![
            (Text::DetailFieldReligion, "catholic", ink, false),
            (Text::DetailFieldCulture, "-", ink, false),
        ],
    );
    expect(
        2,
        vec![
            (Text::DetailFieldRawMaterial, "wheat", CellTone::Food, false),
            (Text::DetailFieldModifier, "-", ink, false),
        ],
    );
    expect(
        3,
        vec![
            (Text::DetailFieldCoastal, "Yes", ink, false),
            (Text::DetailFieldConnectedSea, "baltic_sea", ink, true),
            (Text::DetailFieldHarbor, "0.50", ink, true),
            (Text::DetailFieldMovement, "-1.00, 1.25", ink, true),
        ],
    );
    expect(
        4,
        vec![
            (Text::DetailFieldRiver, "Stream, level 2", ink, false),
            (Text::DetailFieldRiverBonus, "+20%", ink, true),
            (Text::DetailFieldRiverSource, "Yes", ink, false),
            (Text::DetailFieldRiverConfluence, "No", ink, false),
        ],
    );
    expect(
        5,
        vec![
            (Text::DetailFieldCapacityTotal, "1,234,567", ink, true),
            (Text::DetailFieldCapacityVegetation, "1,200", ink, true),
            (Text::DetailFieldCapacityEquator, "34,000", ink, true),
            (Text::DetailFieldCapacityModifier, "-0.25%", ink, true),
        ],
    );
    assert_eq!(group(&detail, 5).and_then(|g| g.note), Some(Text::DetailCapacityNote));
}

#[test]
fn missing_values_show_the_placeholder() {
    let data = dataset();
    let Some(detail) = detail_of(&data, 1) else {
        assert!(detail_of(&data, 1).is_some());
        return;
    };
    assert_eq!(detail.name, "Baltic Sea");
    assert_eq!(detail.hex, "#0000FF");
    let values = |index: usize| group(&detail, index).map(|g| rows(g).iter().map(|row| row.1.to_owned()).collect::<Vec<_>>());
    assert_eq!(values(0), Some(vec!["Sea".into(), "ocean".into(), "-".into(), "-".into()]));
    assert_eq!(values(1), Some(vec!["-".to_owned(), "-".to_owned()]));
    assert_eq!(values(2), Some(vec!["-".to_owned(), "-".to_owned()]));
    assert_eq!(values(3), Some(vec!["No".into(), "-".into(), "-".into(), "-".into()]));
    // Without a river only the river row appears.
    assert_eq!(values(4), Some(vec!["Missing".to_owned()]));
    assert_eq!(values(5), Some(vec!["-".to_owned(); 4]));
}

#[test]
fn river_tiers_and_capacity_modifiers_format_exactly() {
    let mut data = dataset();
    for (value, expected) in [
        (150, "+1.50%"),
        (0, "+0.00%"),
        (5, "+0.05%"),
        (-5, "-0.05%"),
        (-100, "-1.00%"),
        (i16::MIN, "-327.68%"),
        (i16::MAX, "+327.67%"),
    ] {
        if let Some(capacity) = data
            .stored
            .locations
            .get_mut(2)
            .and_then(|record| record.static_population_capacity.as_mut())
        {
            capacity.modifier_basis_points = value;
        }
        let detail = detail_of(&data, 2);
        let modifier = detail
            .as_ref()
            .and_then(|detail| group(detail, 5))
            .and_then(|group| group.fields.last())
            .map(|field| field.value.as_str());
        assert_eq!(modifier, Some(expected), "{value}");
    }
    let river = |location: u32| {
        detail_of(&data, location)
            .and_then(|detail| detail.groups.get(4).map(rows_owned))
    };
    assert_eq!(
        river(2),
        Some(vec!["Major river, level 5".to_owned(), "+50%".into(), "No".into(), "Yes".into()])
    );
    assert_eq!(
        river(3).map(|values| values.into_iter().take(2).collect::<Vec<_>>()),
        Some(vec!["Unknown, level 9".to_owned(), "+90%".into()])
    );
}

fn rows_owned(group: &DetailGroup) -> Vec<String> {
    group.fields.iter().map(|field| field.value.clone()).collect()
}

#[test]
fn precious_and_plain_materials_carry_their_tone() {
    let data = dataset();
    let tone = |location: u32| {
        detail_of(&data, location)
            .and_then(|detail| detail.groups.get(2).and_then(|g| g.fields.first().map(|f| f.tone)))
    };
    assert_eq!(tone(0), Some(CellTone::Food));
    assert_eq!(tone(2), Some(CellTone::Precious));
    assert_eq!(tone(3), Some(CellTone::Ink));
    assert_eq!(tone(1), Some(CellTone::Ink));
}

#[test]
fn unknown_locations_have_no_detail() {
    let data = dataset();
    assert!(detail_of(&data, 4).is_none());
    assert!(detail_of(&data, u32::MAX).is_none());
}

#[test]
fn the_active_game_language_names_the_detail() {
    let data = dataset();
    let pack = korean_pack(&data);
    assert!(pack.is_some(), "the synthetic pack must validate");
    let Some(pack) = pack else { return };
    let detail = build_detail(Labels::new(&data, Some(&pack)), Locale::EnUs, LocationId(0));
    assert_eq!(detail.as_ref().map(|d| d.name.as_str()), Some("스톡홀름"));
    assert_eq!(detail.as_ref().map(|d| d.key.as_str()), Some("stockholm"));
    let first = |index: usize| {
        detail
            .as_ref()
            .and_then(|d| d.groups.get(index))
            .and_then(|g| g.fields.get(1))
            .map(|f| f.value.clone())
    };
    assert_eq!(first(0).as_deref(), Some("언덕"));
}
