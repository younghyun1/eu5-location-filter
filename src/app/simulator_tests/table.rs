//! The result grid and its toolbar: sorting by a header, hiding and showing columns, selecting
//! a row, and the grid's own keys.

use iced::event;
use iced::keyboard::{Key, key::Named};

use super::fixture::{LAND_BY_IDENTIFIER, LAND_BY_NAME};
use super::harness::{Harness, WIDE};
use super::selectors::{grid_shows, row_y, text_in, text_selector};
use crate::AppError;
use crate::app::i18n::{Locale, Text, t};
use crate::app::ids;
use crate::app::message::{
    ColumnMessage, FilterMessage, GridMessage, Message, OverlayMessage, PaneMessage,
    SelectionSource,
};
use crate::app::state::Surface;
use crate::app::view_model::{ColumnId, SortState};
use crate::filter::SortField;
use crate::model::LocationId;

/// Brevik, the second land location by name.
const BREVIK: LocationId = LocationId(3);

/// English title of `column`.
fn title(column: ColumnId) -> &'static str {
    t(Locale::EnUs, column.title())
}

#[test]
fn clicking_a_header_sorts_and_clicking_it_again_reverses() -> Result<(), AppError> {
    let mut harness = Harness::new(WIDE)?;
    assert_eq!(harness.results()?, LAND_BY_NAME.as_slice());

    let messages = harness.drive(|ui| {
        let header = text_selector(ui, ids::GRID, title(ColumnId::Identifier))?;
        let _target = ui.click(header)?;
        Ok(())
    })?;
    assert!(
        messages
            .iter()
            .any(|message| matches!(message, Message::Grid(GridMessage::Sort(ColumnId::Identifier)))),
        "{messages:?}"
    );
    let expected = SortState {
        field: SortField::Identifier,
        ascending: true,
    };
    assert_eq!(harness.app.sort, expected);
    assert_eq!(harness.results()?, LAND_BY_IDENTIFIER.as_slice());
    let mut ui = harness.ui();
    assert!(row_y(&mut ui, "Drovnik")? < row_y(&mut ui, "Aldoria")?, "Drovnik leads by identifier");
    drop(ui);

    for ascending in [true, false] {
        harness.drive(|ui| {
            let header = text_selector(ui, ids::GRID, title(ColumnId::Name))?;
            let _target = ui.click(header)?;
            Ok(())
        })?;
        let expected = SortState {
            field: SortField::Name,
            ascending,
        };
        assert_eq!(harness.app.sort, expected);
    }
    let mut descending = LAND_BY_NAME;
    descending.reverse();
    assert_eq!(harness.results()?, descending.as_slice());
    let mut ui = harness.ui();
    assert!(row_y(&mut ui, "Fenmark")? < row_y(&mut ui, "Aldoria")?, "Fenmark leads when descending");
    Ok(())
}

/// Opens the column menu from its toolbar button.
fn open_column_menu(harness: &mut Harness) -> Result<(), AppError> {
    let messages = harness.drive(|ui| {
        let _button = ui.click(ids::COLUMNS_BUTTON)?;
        Ok(())
    })?;
    assert!(
        messages.iter().any(|message| matches!(
            message,
            Message::Overlay(OverlayMessage::Toggle(Surface::ColumnMenu))
        )),
        "{messages:?}"
    );
    assert_eq!(harness.app.overlay.surface, Some(Surface::ColumnMenu));
    Ok(())
}

/// Clicks the menu row of `column`.
fn click_column_row(harness: &mut Harness, column: ColumnId) -> Result<Vec<Message>, AppError> {
    harness.drive(|ui| {
        let row = text_selector(ui, ids::COLUMN_MENU_LIST, title(column))?;
        let _target = ui.click(row)?;
        Ok(())
    })
}

#[test]
fn a_column_can_be_hidden_and_shown_from_the_menu() -> Result<(), AppError> {
    let mut harness = Harness::new(WIDE)?;
    let column = ColumnId::Topography;
    assert!(grid_shows(&mut harness.ui(), title(column)));

    open_column_menu(&mut harness)?;
    let messages = click_column_row(&mut harness, column)?;
    assert!(
        messages.iter().any(|message| matches!(
            message,
            Message::Column(ColumnMessage::SetVisible {
                column: ColumnId::Topography,
                visible: false,
            })
        )),
        "{messages:?}"
    );
    assert!(!harness.app.columns.get(column).visible);
    harness.send(Message::Overlay(OverlayMessage::Close));
    assert_eq!(harness.app.overlay.surface, None);
    let mut ui = harness.ui();
    assert!(!grid_shows(&mut ui, title(column)), "the hidden column has no header");
    assert!(grid_shows(&mut ui, title(ColumnId::Vegetation)), "its neighbours stay");
    drop(ui);

    open_column_menu(&mut harness)?;
    click_column_row(&mut harness, column)?;
    harness.send(Message::Overlay(OverlayMessage::Close));
    assert!(harness.app.columns.get(column).visible);
    assert!(grid_shows(&mut harness.ui(), title(column)));
    Ok(())
}

#[test]
fn selecting_a_row_fills_the_detail_pane() -> Result<(), AppError> {
    let mut harness = Harness::new(WIDE)?;
    let empty = t(Locale::EnUs, Text::DetailEmptyTitle);
    assert!(harness.ui().find(empty).is_ok(), "no selection shows the empty state");

    let messages = harness.drive(|ui| {
        let cell = text_selector(ui, ids::GRID, "Brevik")?;
        let _target = ui.click(cell)?;
        Ok(())
    })?;
    assert!(
        messages.iter().any(|message| matches!(
            message,
            Message::Grid(GridMessage::Select { id, source: SelectionSource::Pointer }) if *id == BREVIK
        )),
        "{messages:?}"
    );
    assert_eq!(harness.app.selection, Some(BREVIK));
    assert_eq!(harness.app.detail.as_ref().map(|detail| detail.name.as_str()), Some("Brevik"));

    let mut ui = harness.ui();
    assert!(ui.find(empty).is_err());
    assert!(text_in(&mut ui, ids::DETAIL_SCROLL, "Brevik").is_ok(), "the name heading");
    assert!(text_in(&mut ui, ids::DETAIL_SCROLL, "quill").is_ok(), "the identifier");
    drop(ui);

    harness.send(Message::Filter(FilterMessage::Search("ald".to_owned())));
    assert_eq!(harness.app.selection, None, "a selection that leaves the results is dropped");
    assert!(harness.ui().find(empty).is_ok());
    Ok(())
}

/// Selects Brevik through the application so the next interface builds with it selected.
fn select_brevik(harness: &mut Harness) {
    harness.send(Message::Grid(GridMessage::Select {
        id: BREVIK,
        source: SelectionSource::Keyboard,
    }));
}

#[test]
fn arrow_down_in_the_focused_grid_selects_the_next_row() -> Result<(), AppError> {
    let mut harness = Harness::new(WIDE)?;
    select_brevik(&mut harness);

    let messages = harness.interact(|ui| {
        let cell = text_selector(ui, ids::GRID, "Brevik")?;
        let _target = ui.click(cell)?;
        let _status = ui.tap_key(Key::Named(Named::ArrowDown));
        Ok(())
    })?;

    let last = messages.last();
    assert!(
        matches!(
            last,
            Some(Message::Grid(GridMessage::Select { id, source: SelectionSource::Keyboard }))
                if *id == LAND_BY_NAME[2]
        ),
        "{messages:?}"
    );
    Ok(())
}

#[test]
fn the_grid_leaves_tab_to_the_application() -> Result<(), AppError> {
    let harness = Harness::new(WIDE)?;
    let mut status = event::Status::Captured;
    harness.interact(|ui| {
        let cell = text_selector(ui, ids::GRID, "Brevik")?;
        let _target = ui.click(cell)?;
        status = ui.tap_key(Key::Named(Named::Tab));
        Ok(())
    })?;
    assert_eq!(status, event::Status::Ignored);
    Ok(())
}

#[test]
fn space_on_the_selected_row_shows_a_hidden_detail_pane() -> Result<(), AppError> {
    let mut harness = Harness::new(WIDE)?;
    select_brevik(&mut harness);
    harness.send(Message::Pane(PaneMessage::SetDetailHidden(true)));
    assert!(harness.app.panes.detail_hidden);

    let messages = harness.drive(|ui| {
        let cell = text_selector(ui, ids::GRID, "Brevik")?;
        let _target = ui.click(cell)?;
        let _status = ui.tap_key(Key::Named(Named::Space));
        Ok(())
    })?;

    assert!(
        messages
            .iter()
            .any(|message| matches!(message, Message::Grid(GridMessage::ShowDetail(id)) if *id == BREVIK)),
        "{messages:?}"
    );
    assert!(!harness.app.panes.detail_hidden);
    Ok(())
}
