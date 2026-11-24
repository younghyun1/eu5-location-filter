//! Column defaults, visibility, width clamping and the 1,346 px total width.

use super::{Column, ColumnId, ColumnRows, Columns};
use crate::app::i18n::{Locale, Text, t};
use crate::app::layout::{COLUMN_MAX_WIDTH, COLUMN_MIN_WIDTH, GRID_INLINE_INSET};
use crate::app::widget::ChecklistModel;
use crate::filter::SortField;

#[test]
fn default_total_width_matches_the_ported_value() {
    let columns = Columns::default();
    assert_eq!(columns.total_width(), 1_346.0);
    assert_eq!(columns.total_width() - GRID_INLINE_INSET, 1_336.0);
}

#[test]
fn defaults_follow_the_contract_table() {
    let columns = Columns::default();
    let visible: Vec<ColumnId> = columns.visible().map(|column| column.id).collect();
    assert_eq!(
        visible,
        [
            ColumnId::Color,
            ColumnId::Name,
            ColumnId::Identifier,
            ColumnId::Kind,
            ColumnId::Topography,
            ColumnId::Vegetation,
            ColumnId::Climate,
            ColumnId::RawMaterial,
            ColumnId::RiverLevel,
            ColumnId::Harbor,
            ColumnId::StaticCapacity,
        ]
    );
    assert_eq!(columns.visible_count(), 11);
    assert_eq!(columns.all().len(), ColumnId::COUNT);
    for (position, column) in columns.all().iter().enumerate() {
        assert_eq!(column.id.index(), position);
        assert_eq!(ColumnId::ALL.get(position), Some(&column.id));
        assert_eq!(column.width, column.id.default_width());
        assert_eq!(column.visible, column.id.default_visible());
    }
}

#[test]
fn keys_are_unique_and_round_trip() {
    for column in ColumnId::ALL {
        assert_eq!(ColumnId::from_key(column.key()), Some(column));
    }
    assert_eq!(ColumnId::from_key("nope"), None);
    assert_eq!(ColumnId::from_key(""), None);
    let mut keys: Vec<&str> = ColumnId::ALL.iter().map(|column| column.key()).collect();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(keys.len(), ColumnId::COUNT);
}

#[test]
fn widths_are_inside_the_resize_limits() {
    for column in ColumnId::ALL {
        let width = column.default_width();
        assert!(
            (COLUMN_MIN_WIDTH..=COLUMN_MAX_WIDTH).contains(&width),
            "{column:?}"
        );
    }
}

#[test]
fn width_changes_clamp_and_recompute_the_total() {
    let mut columns = Columns::default();
    columns.set_width(ColumnId::Name, 400.0);
    assert_eq!(columns.get(ColumnId::Name).width, 400.0);
    assert_eq!(columns.total_width(), 1_346.0 + 190.0);
    columns.set_width(ColumnId::Name, 10_000.0);
    assert_eq!(columns.get(ColumnId::Name).width, COLUMN_MAX_WIDTH);
    columns.set_width(ColumnId::Name, -5.0);
    assert_eq!(columns.get(ColumnId::Name).width, COLUMN_MIN_WIDTH);
}

#[test]
fn hidden_columns_do_not_count_toward_the_width() {
    let mut columns = Columns::default();
    let before = columns.total_width();
    columns.set_width(ColumnId::Continent, 300.0);
    assert_eq!(columns.total_width(), before);
    assert!(columns.set_visible(ColumnId::Continent, true));
    assert_eq!(columns.total_width(), before + 300.0);
    assert!(columns.set_visible(ColumnId::Continent, false));
    assert_eq!(columns.total_width(), before);
}

#[test]
fn the_last_visible_column_cannot_be_hidden() {
    let mut columns = Columns::default();
    for column in ColumnId::ALL {
        if column != ColumnId::Name {
            assert!(columns.set_visible(column, false), "{column:?}");
        }
    }
    assert_eq!(columns.visible_count(), 1);
    assert!(!columns.set_visible(ColumnId::Name, false));
    assert!(columns.get(ColumnId::Name).visible);
    assert_eq!(columns.visible_count(), 1);
    // Showing another column makes the first one hideable again.
    assert!(columns.set_visible(ColumnId::Kind, true));
    assert!(columns.set_visible(ColumnId::Name, false));
    assert_eq!(
        columns
            .visible()
            .map(|column| column.id)
            .collect::<Vec<_>>(),
        [ColumnId::Kind]
    );
}

#[test]
fn hiding_a_hidden_column_is_accepted() {
    let mut columns = Columns::default();
    assert!(columns.set_visible(ColumnId::Region, false));
    assert!(!columns.get(ColumnId::Region).visible);
    // A hidden column may be requested hidden even when only one is visible.
    for column in ColumnId::ALL {
        if column != ColumnId::Color {
            let _ = columns.set_visible(column, false);
        }
    }
    assert!(columns.set_visible(ColumnId::Region, false));
}

#[test]
fn get_returns_the_stored_state() {
    let mut columns = Columns::default();
    columns.set_width(ColumnId::Rgb, 222.0);
    let rgb = columns.get(ColumnId::Rgb);
    assert_eq!(
        rgb,
        Column {
            id: ColumnId::Rgb,
            width: 222.0,
            visible: false,
        }
    );
}

#[test]
fn alignment_and_fonts_follow_the_contract() {
    use super::{CellAlign, CellFont};
    let end = [
        ColumnId::RiverLevel,
        ColumnId::Harbor,
        ColumnId::MovementX,
        ColumnId::MovementY,
        ColumnId::StaticCapacity,
        ColumnId::EquatorCapacity,
    ];
    for column in ColumnId::ALL {
        let expected = if end.contains(&column) {
            CellAlign::End
        } else {
            CellAlign::Start
        };
        assert_eq!(column.align(), expected, "{column:?}");
    }
    assert_eq!(ColumnId::Color.font(), CellFont::Swatch);
    assert_eq!(ColumnId::Name.font(), CellFont::SansMedium);
    assert_eq!(ColumnId::Identifier.font(), CellFont::Mono);
    assert_eq!(ColumnId::Rgb.font(), CellFont::Mono);
    assert_eq!(ColumnId::StaticCapacity.font(), CellFont::Mono);
    assert_eq!(ColumnId::Kind.font(), CellFont::Sans);
    assert_eq!(ColumnId::RawMaterial.font(), CellFont::Sans);
}

#[test]
fn column_menu_rows_mirror_visibility() {
    let mut columns = Columns::default();
    assert!(columns.set_visible(ColumnId::Continent, true));
    let rows = ColumnRows {
        columns: &columns,
        locale: Locale::EnUs,
    };
    assert_eq!(rows.row_count(), ColumnId::COUNT);
    for (row, column) in ColumnId::ALL.into_iter().enumerate() {
        assert_eq!(rows.label(row), t(Locale::EnUs, column.title()));
        assert_eq!(rows.is_checked(row), columns.get(column).visible);
        assert_eq!(rows.option(row), u32::try_from(row).unwrap_or(u32::MAX));
    }
    assert_eq!(rows.label(ColumnId::COUNT), "");
    assert!(!rows.is_checked(ColumnId::COUNT));
    assert_eq!(rows.label(0), t(Locale::EnUs, Text::ColumnColor));
}

#[test]
fn sort_fields_cover_every_field_and_alias_rgb_to_color() {
    assert_eq!(ColumnId::Rgb.sort_field(), SortField::Color);
    assert_eq!(ColumnId::Color.sort_field(), SortField::Color);
    for field in SortField::ALL {
        assert!(
            ColumnId::ALL
                .iter()
                .any(|column| column.sort_field() == field),
            "{field:?}"
        );
    }
}
