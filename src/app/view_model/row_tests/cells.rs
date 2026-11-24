//! Cell text for every column on four locations, tones and buffer reuse.

use crate::app::i18n::Locale;
use crate::app::theme::{Mode, PALETTE_LIGHT, Palette};
use crate::app::view_model::options::test_data::{dataset, english_context, korean_pack};
use crate::app::view_model::{CellTone, ColumnId, RowContext, tone_color, write_cell};
use crate::lang_pack::Labels;
use crate::model::LocationId;

/// Text and tone of one cell of a location in the fixture, English.
fn cell(location: u32, column: ColumnId) -> (String, CellTone) {
    let data = dataset();
    let context = english_context(&data);
    let mut buffer = String::from("stale text that must be replaced");
    let tone = match data.location(LocationId(location)) {
        Some(record) => write_cell(&mut buffer, &context, record, column),
        None => CellTone::Muted,
    };
    (buffer, tone)
}

/// Asserts the text of every column of a location against `expected` in column order.
fn assert_row(location: u32, expected: [&str; ColumnId::COUNT]) {
    for (column, text) in ColumnId::ALL.into_iter().zip(expected) {
        assert_eq!(
            cell(location, column).0,
            text,
            "location {location} {column:?}"
        );
    }
}

#[test]
fn stockholm_has_every_value() {
    assert_row(
        0,
        [
            "#1A2B3C",
            "Stockholm",
            "stockholm",
            "Land",
            "hills",
            "forest",
            "boreal",
            "Europe",
            "Kouřim",
            "Ra's al-'Ain",
            "Москва",
            "東京",
            "catholic",
            "-",
            "wheat",
            "-",
            "rgb(26, 43, 60)",
            "Yes",
            "Present",
            "L2 +20%",
            "0.50",
            "Present",
            "-1.00",
            "1.25",
            "1,234,567",
            "34,000",
        ],
    );
}

#[test]
fn the_baltic_has_only_required_values() {
    assert_row(
        1,
        [
            "#0000FF",
            "Baltic Sea",
            "baltic_sea",
            "Sea",
            "ocean",
            "-",
            "-",
            "Europe",
            "Kouřim",
            "Ra's al-'Ain",
            "Москва",
            "東京",
            "-",
            "-",
            "-",
            "-",
            "rgb(0, 0, 255)",
            "No",
            "Missing",
            "-",
            "-",
            "Missing",
            "-",
            "-",
            "-",
            "-",
        ],
    );
}

#[test]
fn zero_capacity_and_top_river_tier_format_plainly() {
    assert_eq!(cell(2, ColumnId::StaticCapacity).0, "0");
    assert_eq!(cell(2, ColumnId::EquatorCapacity).0, "0");
    assert_eq!(cell(2, ColumnId::RiverLevel).0, "L5 +50%");
    assert_eq!(cell(2, ColumnId::RawMaterial).0, "goods_gold");
    assert_eq!(cell(3, ColumnId::RiverLevel).0, "L9 +90%");
    assert_eq!(cell(3, ColumnId::RiverPresence).0, "Present");
    assert_eq!(cell(3, ColumnId::StaticCapacity).0, "-");
}

#[test]
fn tones_follow_the_contract() {
    for column in ColumnId::ALL {
        let expected = match column {
            ColumnId::RawMaterial => CellTone::Food,
            ColumnId::RiverLevel => CellTone::River,
            _ => CellTone::Ink,
        };
        assert_eq!(cell(0, column).1, expected, "wheat location {column:?}");
    }
    assert_eq!(cell(2, ColumnId::RawMaterial).1, CellTone::Precious);
    assert_eq!(cell(3, ColumnId::RawMaterial).1, CellTone::Ink);
    assert_eq!(cell(1, ColumnId::RawMaterial).1, CellTone::Ink);
    assert_eq!(cell(1, ColumnId::RiverLevel).1, CellTone::Ink);
    assert_eq!(cell(3, ColumnId::RiverLevel).1, CellTone::River);
}

#[test]
fn every_food_and_precious_good_is_classified() {
    for (material, tone) in [
        ("wheat", CellTone::Food),
        ("fur", CellTone::Food),
        ("goods_gold", CellTone::Precious),
        ("silver", CellTone::Precious),
        ("iron", CellTone::Ink),
    ] {
        let mut data = dataset();
        let symbol =
            crate::model::SymbolId(u32::try_from(data.stored.dictionary.len()).unwrap_or(0));
        data.stored.dictionary.push(material.to_owned());
        if let Some(record) = data.stored.locations.first_mut() {
            record.raw_material = Some(symbol);
        }
        let context = english_context(&data);
        let mut buffer = String::new();
        let observed = data
            .location(LocationId(0))
            .map(|record| write_cell(&mut buffer, &context, record, ColumnId::RawMaterial));
        assert_eq!(observed, Some(tone), "{material}");
        assert_eq!(buffer, material);
    }
}

#[test]
fn the_buffer_is_cleared_and_reused() {
    let data = dataset();
    let context = english_context(&data);
    let mut buffer = String::new();
    let Some(record) = data.location(LocationId(0)) else {
        assert!(data.location(LocationId(0)).is_some());
        return;
    };
    let _ = write_cell(&mut buffer, &context, record, ColumnId::Rgb);
    let capacity = buffer.capacity();
    for column in ColumnId::ALL {
        let _ = write_cell(&mut buffer, &context, record, column);
        assert!(!buffer.contains("stale"), "{column:?}");
        assert!(buffer.capacity() >= capacity, "{column:?}");
    }
    let _ = write_cell(&mut buffer, &context, record, ColumnId::Kind);
    assert_eq!(buffer, "Land");
}

#[test]
fn text_is_identical_across_repeated_calls() {
    for column in ColumnId::ALL {
        assert_eq!(cell(0, column), cell(0, column), "{column:?}");
    }
}

#[test]
fn game_language_changes_names_and_labels_but_not_numbers() {
    let data = dataset();
    let pack = korean_pack(&data);
    assert!(pack.is_some(), "the synthetic pack must validate");
    let Some(pack) = pack else { return };
    let context = RowContext {
        labels: Labels::new(&data, Some(&pack)),
        locale: Locale::EnUs,
    };
    let mut buffer = String::new();
    let mut read = |column| {
        data.location(LocationId(0))
            .map(|record| write_cell(&mut buffer, &context, record, column))
            .map(|_| buffer.clone())
    };
    assert_eq!(read(ColumnId::Name).as_deref(), Some("스톡홀름"));
    assert_eq!(read(ColumnId::Topography).as_deref(), Some("언덕"));
    assert_eq!(read(ColumnId::RawMaterial).as_deref(), Some("밀"));
    // English fallback for unlabelled symbols, identifier unchanged, numbers invariant.
    assert_eq!(read(ColumnId::Vegetation).as_deref(), Some("forest"));
    assert_eq!(read(ColumnId::Identifier).as_deref(), Some("stockholm"));
    assert_eq!(read(ColumnId::StaticCapacity).as_deref(), Some("1,234,567"));
    assert_eq!(read(ColumnId::Harbor).as_deref(), Some("0.50"));
}

#[test]
fn numbers_are_invariant_in_every_locale() {
    let data = dataset();
    for locale in Locale::ALL {
        let context = RowContext {
            labels: Labels::new(&data, None),
            locale,
        };
        let mut buffer = String::new();
        for (column, expected) in [
            (ColumnId::StaticCapacity, "1,234,567"),
            (ColumnId::EquatorCapacity, "34,000"),
            (ColumnId::Harbor, "0.50"),
            (ColumnId::MovementX, "-1.00"),
            (ColumnId::Color, "#1A2B3C"),
            (ColumnId::Identifier, "stockholm"),
        ] {
            let tone = data
                .location(LocationId(0))
                .map(|record| write_cell(&mut buffer, &context, record, column));
            assert_eq!(tone, Some(CellTone::Ink), "{locale:?} {column:?}");
            assert_eq!(buffer, expected, "{locale:?} {column:?}");
        }
    }
}

#[test]
fn tone_colors_come_from_the_palette() {
    let palette: Palette = PALETTE_LIGHT;
    assert_eq!(palette.mode, Mode::Light);
    assert_eq!(tone_color(palette, CellTone::Ink), palette.ink);
    assert_eq!(tone_color(palette, CellTone::Muted), palette.ink_muted);
    assert_eq!(tone_color(palette, CellTone::Food), palette.ok);
    assert_eq!(tone_color(palette, CellTone::Precious), palette.accent);
    assert_eq!(tone_color(palette, CellTone::River), palette.accent_strong);
}
