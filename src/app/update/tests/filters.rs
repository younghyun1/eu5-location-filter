//! Filter, field validation, sort, selection and column tests.

use super::fixture::{LAND, PEAKS, SEA, app};
use crate::AppError;
use crate::app::context::Platform;
use crate::app::i18n::Text;
use crate::app::layout::COLUMN_MAX_WIDTH;
use crate::app::message::{
    ColumnMessage, FilterMessage, GridMessage, Message, SelectionSource,
};
use crate::app::update::update;
use crate::app::view_model::view_state::MAX_SEARCH_BYTES;
use crate::app::view_model::{Bound, ColumnId, FacetId, RangeField};
use crate::filter::SortField;
use crate::model::{LocationKind, MapColor};

/// Sends one filter message.
fn send(app: &mut crate::app::state::App, message: FilterMessage) {
    let _task = update(app, Message::Filter(message));
}

#[test]
fn ready_app_starts_on_land_only() -> Result<(), AppError> {
    let app = app(Platform::Desktop)?;
    let results = app.data.as_ref().map(|data| data.results.clone());
    assert_eq!(results, Some(vec![LAND]));
    assert!(app.filters.kinds.contains(&LocationKind::Land));
    Ok(())
}

#[test]
fn selection_survives_only_while_in_the_results() -> Result<(), AppError> {
    let mut app = app(Platform::Desktop)?;
    let _task = update(
        &mut app,
        Message::Grid(GridMessage::Select {
            id: LAND,
            source: SelectionSource::Pointer,
        }),
    );
    assert_eq!(app.selection, Some(LAND));
    send(&mut app, FilterMessage::Search("zzzz".to_owned()));
    assert_eq!(app.selection, None);
    assert!(app.detail.is_none());
    send(&mut app, FilterMessage::Search(String::new()));
    assert_eq!(app.selection, None);
    Ok(())
}

#[test]
fn selecting_a_location_outside_the_results_is_ignored() -> Result<(), AppError> {
    let mut app = app(Platform::Desktop)?;
    for id in [SEA, PEAKS] {
        let _task = update(
            &mut app,
            Message::Grid(GridMessage::Select {
                id,
                source: SelectionSource::Keyboard,
            }),
        );
    }
    assert_eq!(app.selection, None);
    Ok(())
}

#[test]
fn search_is_truncated_on_a_character_boundary() -> Result<(), AppError> {
    let mut app = app(Platform::Desktop)?;
    send(&mut app, FilterMessage::Search("가".repeat(100)));
    assert!(app.filters.search.len() <= MAX_SEARCH_BYTES);
    assert!(app.filters.search.chars().all(|character| character == '가'));
    Ok(())
}

#[test]
fn invalid_colour_keeps_the_filter_and_shows_an_error() -> Result<(), AppError> {
    let mut app = app(Platform::Desktop)?;
    send(&mut app, FilterMessage::Rgb("#1a2b3c".to_owned()));
    assert_eq!(app.filters.rgb, Some(MapColor(0x1A_2B_3C)));
    assert_eq!(app.fields.rgb.error, None);
    send(&mut app, FilterMessage::Rgb("xyz".to_owned()));
    assert_eq!(app.filters.rgb, Some(MapColor(0x1A_2B_3C)));
    assert_eq!(app.fields.rgb.error, Some(Text::ValidationRgb));
    assert_eq!(app.fields.rgb.text, "xyz");
    send(&mut app, FilterMessage::Rgb("  ".to_owned()));
    assert_eq!(app.filters.rgb, None);
    assert_eq!(app.fields.rgb.error, None);
    Ok(())
}

#[test]
fn range_fields_distinguish_malformed_and_infinite_numbers() -> Result<(), AppError> {
    let mut app = app(Platform::Desktop)?;
    let cases = [
        ("abc", Some(Text::ValidationNumber), None),
        ("inf", Some(Text::ValidationNumberFinite), None),
        ("1e39", Some(Text::ValidationNumberFinite), None),
        (" 0.25 ", None, Some(0.25)),
        ("", None, None),
    ];
    for (text, error, min) in cases {
        send(
            &mut app,
            FilterMessage::Range {
                field: RangeField::Harbor,
                bound: Bound::Min,
                text: text.to_owned(),
            },
        );
        assert_eq!(app.fields.range(RangeField::Harbor, Bound::Min).error, error);
        if error.is_none() {
            assert_eq!(app.filters.harbor_range.min, min);
        }
    }
    Ok(())
}

#[test]
fn clear_all_restores_defaults_and_empties_fields() -> Result<(), AppError> {
    let mut app = app(Platform::Desktop)?;
    send(&mut app, FilterMessage::Search("stock".to_owned()));
    send(&mut app, FilterMessage::Rgb("ff".to_owned()));
    send(&mut app, FilterMessage::ShowImpassable(false));
    send(
        &mut app,
        FilterMessage::Range {
            field: RangeField::MovementY,
            bound: Bound::Max,
            text: "nope".to_owned(),
        },
    );
    send(
        &mut app,
        FilterMessage::OptionQuery {
            facet: FacetId::Culture,
            query: "sw".to_owned(),
        },
    );
    send(&mut app, FilterMessage::ClearAll);
    assert!(app.filters.search.is_empty());
    assert_eq!(app.filters.rgb, None);
    assert!(app.filters.show_impassable);
    assert_eq!(app.filters.kinds.len(), 1);
    assert!(app.fields.rgb.text.is_empty());
    assert!(app.fields.range(RangeField::MovementY, Bound::Max).text.is_empty());
    assert!(app.facets.iter().all(|group| group.query.is_empty()));
    Ok(())
}

#[test]
fn folding_changes_only_that_group() -> Result<(), AppError> {
    let mut app = app(Platform::Desktop)?;
    send(
        &mut app,
        FilterMessage::Fold {
            facet: FacetId::Region,
            folded: false,
        },
    );
    for group in &app.facets {
        let expected = group.facet != FacetId::Region && group.facet.starts_folded();
        assert_eq!(group.folded, expected);
    }
    Ok(())
}

#[test]
fn sorting_the_same_column_reverses_it() -> Result<(), AppError> {
    let mut app = app(Platform::Desktop)?;
    let _task = update(&mut app, Message::Grid(GridMessage::Sort(ColumnId::Name)));
    assert_eq!(app.sort.field, SortField::Name);
    assert!(!app.sort.ascending);
    let _task = update(&mut app, Message::Grid(GridMessage::Sort(ColumnId::Harbor)));
    assert_eq!(app.sort.field, SortField::HarborSuitability);
    assert!(app.sort.ascending);
    Ok(())
}

#[test]
fn column_widths_are_clamped_and_non_finite_widths_dropped() -> Result<(), AppError> {
    let mut app = app(Platform::Desktop)?;
    let before = app.columns.get(ColumnId::Name).width;
    let resize = |width| Message::Grid(GridMessage::ResizeColumn {
        column: ColumnId::Name,
        width,
    });
    let _task = update(&mut app, resize(f32::NAN));
    assert_eq!(app.columns.get(ColumnId::Name).width, before);
    let _task = update(&mut app, resize(10_000.0));
    assert_eq!(app.columns.get(ColumnId::Name).width, COLUMN_MAX_WIDTH);
    Ok(())
}

#[test]
fn the_last_visible_column_cannot_be_hidden() -> Result<(), AppError> {
    let mut app = app(Platform::Desktop)?;
    for column in ColumnId::ALL {
        let _task = update(
            &mut app,
            Message::Column(ColumnMessage::SetVisible {
                column,
                visible: false,
            }),
        );
    }
    assert_eq!(app.columns.visible_count(), 1);
    Ok(())
}
