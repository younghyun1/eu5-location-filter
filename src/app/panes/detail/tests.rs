//! Unit tests of the detail pane builders.

use iced::Color;

use super::rows::{breadcrumb_levels, field_padding};
use super::{swatch_color, view};
use crate::app::i18n::Text;
use crate::app::layout::DETAIL_FIELD_MIN_HEIGHT;
use crate::app::panes::overlays::kit::test_support::contexts;
use crate::app::view_model::{CellTone, DetailField, DetailGroup, LocationDetail};
use crate::model::{LocationId, MapColor};

fn sample() -> LocationDetail {
    LocationDetail {
        id: LocationId(7),
        color: MapColor(0x12_AB_EF),
        hex: "#12ABEF".to_owned(),
        rgb: "18, 171, 239".to_owned(),
        name: "Stockholm".to_owned(),
        key: "stockholm".to_owned(),
        breadcrumb: vec![
            "Europe".to_owned(),
            String::new(),
            "Scandinavia".to_owned(),
            "Svealand".to_owned(),
        ],
        groups: vec![
            DetailGroup {
                title: Text::DetailGroupClassification,
                fields: vec![DetailField {
                    label: Text::DetailFieldType,
                    value: "Land".to_owned(),
                    tone: CellTone::Ink,
                    mono: false,
                }],
                note: None,
            },
            DetailGroup {
                title: Text::DetailGroupCapacity,
                fields: vec![DetailField {
                    label: Text::DetailFieldCapacityTotal,
                    value: "1,234".to_owned(),
                    tone: CellTone::Food,
                    mono: true,
                }],
                note: Some(Text::DetailCapacityNote),
            },
        ],
    }
}

#[test]
fn swatch_colour_unpacks_the_channels() {
    assert_eq!(
        swatch_color(MapColor(0xFF_00_80)),
        Color::from_rgb8(255, 0, 128)
    );
    assert_eq!(swatch_color(MapColor(0)), Color::from_rgb8(0, 0, 0));
    // Bits above the 24 packed ones are ignored.
    assert_eq!(
        swatch_color(MapColor(0xAB_01_02_03)),
        Color::from_rgb8(1, 2, 3)
    );
}

#[test]
fn empty_levels_are_skipped() {
    let levels = vec!["A".to_owned(), String::new(), "C".to_owned(), String::new()];
    assert_eq!(breadcrumb_levels(&levels), ["A", "C"]);
    assert!(breadcrumb_levels(&[]).is_empty());
}

#[test]
fn one_line_rows_reach_the_minimum_height() {
    let padding = field_padding();
    assert!(padding > 0.0);
    let line = crate::app::theme::TEXT_BODY * crate::app::theme::LINE_HEIGHT_PROSE;
    assert!((line + 2.0 * padding - DETAIL_FIELD_MIN_HEIGHT).abs() < 0.01);
}

#[test]
fn the_pane_builds_with_and_without_a_selection() {
    let detail = sample();
    for ctx in contexts() {
        for in_drawer in [false, true] {
            let _empty = view(ctx, None, in_drawer);
            let _selected = view(ctx, Some(&detail), in_drawer);
        }
    }
}
