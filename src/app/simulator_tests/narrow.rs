//! The narrow layout: the rail and the detail pane become drawers over a scrim.

use iced::Point;
use iced_test::simulator::click;

use super::fixture::{LAND_BY_NAME, LAND_COUNT, SEA_COUNT};
use super::harness::{Harness, NARROW, WIDE};
use super::selectors::{has_id, text_in, text_selector};
use crate::AppError;
use crate::app::i18n::{Locale, Text, t};
use crate::app::ids;
use crate::app::layout::{Drawer, LayoutMode};
use crate::app::message::{GridMessage, Message, OverlayMessage, SelectionSource};
use crate::app::view_model::FacetId;

/// Whether `messages` hold `OverlayMessage::OpenDrawer(drawer)`.
fn opened(messages: &[Message], drawer: Drawer) -> bool {
    messages.iter().any(|message| {
        matches!(message, Message::Overlay(OverlayMessage::OpenDrawer(opened)) if *opened == drawer)
    })
}

/// Whether `messages` hold `OverlayMessage::CloseDrawer`.
fn closed(messages: &[Message]) -> bool {
    messages
        .iter()
        .any(|message| matches!(message, Message::Overlay(OverlayMessage::CloseDrawer)))
}

#[test]
fn the_narrow_header_replaces_the_docked_panes() -> Result<(), AppError> {
    let harness = Harness::new(NARROW)?;
    assert_eq!(harness.app.layout_mode, LayoutMode::Narrow);
    let mut ui = harness.ui();
    assert!(has_id(&mut ui, ids::FILTERS_BUTTON));
    assert!(has_id(&mut ui, ids::DETAILS_BUTTON));
    assert!(has_id(&mut ui, ids::GRID));
    assert!(!has_id(&mut ui, ids::RAIL_CLEAR_ALL), "no docked rail");
    assert!(!has_id(&mut ui, ids::RAIL_SPLITTER));
    assert!(!has_id(&mut ui, ids::DETAIL_HIDE), "no docked detail pane");
    assert!(
        !has_id(&mut ui, ids::SHOW_DETAILS_BUTTON),
        "the Details button replaces it"
    );
    Ok(())
}

#[test]
fn the_filter_drawer_stays_open_while_filtering_and_closes_on_the_scrim() -> Result<(), AppError> {
    let mut harness = Harness::new(NARROW)?;

    let messages = harness.drive(|ui| {
        let _button = ui.click(ids::FILTERS_BUTTON)?;
        Ok(())
    })?;
    assert!(opened(&messages, Drawer::Filters), "{messages:?}");
    harness.settle();
    assert_eq!(harness.app.open_drawer(), Some(Drawer::Filters));
    assert!(
        has_id(&mut harness.ui(), ids::RAIL_CLEAR_ALL),
        "the drawer holds the rail"
    );

    let sea = t(Locale::EnUs, Text::OptionKindSea);
    harness.drive(|ui| {
        let row = text_selector(ui, ids::facet_list(FacetId::Kind), sea)?;
        let _target = ui.click(row)?;
        Ok(())
    })?;
    assert_eq!(harness.results()?.len(), LAND_COUNT + SEA_COUNT);
    assert_eq!(
        harness.app.open_drawer(),
        Some(Drawer::Filters),
        "filtering keeps the drawer"
    );

    // The drawer covers the inline start; the far end of the window is scrim.
    let messages = harness.interact(|ui| {
        ui.point_at(Point::new(NARROW.width - 10.0, NARROW.height / 2.0));
        let _statuses = ui.simulate(click());
        Ok(())
    })?;
    assert!(closed(&messages), "{messages:?}");
    harness.apply(messages);
    assert_eq!(harness.app.open_drawer(), None);
    harness.settle();
    assert_eq!(
        harness.app.panes.drawer, None,
        "the closed drawer is removed after its slide"
    );
    Ok(())
}

#[test]
fn tapping_a_row_opens_the_detail_drawer_and_its_close_button_closes_it() -> Result<(), AppError> {
    let mut harness = Harness::new(NARROW)?;

    harness.drive(|ui| {
        let cell = text_selector(ui, ids::GRID, "Brevik")?;
        let _target = ui.click(cell)?;
        Ok(())
    })?;
    assert_eq!(harness.app.selection, Some(LAND_BY_NAME[1]));
    harness.settle();
    assert_eq!(harness.app.open_drawer(), Some(Drawer::Details));
    let mut ui = harness.ui();
    assert!(
        text_in(&mut ui, ids::DETAIL_SCROLL, "Brevik").is_ok(),
        "the drawer shows the detail"
    );
    drop(ui);

    let messages = harness.drive(|ui| {
        let _button = ui.click(ids::DETAIL_HIDE)?;
        Ok(())
    })?;
    assert!(closed(&messages), "{messages:?}");
    assert_eq!(harness.app.open_drawer(), None);
    Ok(())
}

#[test]
fn the_details_button_opens_an_empty_detail_drawer() -> Result<(), AppError> {
    let mut harness = Harness::new(NARROW)?;
    let messages = harness.drive(|ui| {
        let _button = ui.click(ids::DETAILS_BUTTON)?;
        Ok(())
    })?;
    assert!(opened(&messages, Drawer::Details), "{messages:?}");
    harness.settle();
    assert!(
        harness
            .ui()
            .find(t(Locale::EnUs, Text::DetailEmptyTitle))
            .is_ok()
    );
    Ok(())
}

#[test]
fn keyboard_selection_does_not_open_the_detail_drawer() -> Result<(), AppError> {
    let mut harness = Harness::new(NARROW)?;
    harness.send(Message::Grid(GridMessage::Select {
        id: LAND_BY_NAME[1],
        source: SelectionSource::Keyboard,
    }));
    assert_eq!(harness.app.selection, Some(LAND_BY_NAME[1]));
    assert_eq!(harness.app.open_drawer(), None);
    Ok(())
}

#[test]
fn widening_the_window_closes_an_open_drawer() -> Result<(), AppError> {
    let mut harness = Harness::new(NARROW)?;
    harness.send(Message::Overlay(OverlayMessage::OpenDrawer(
        Drawer::Filters,
    )));
    harness.settle();
    assert_eq!(harness.app.open_drawer(), Some(Drawer::Filters));

    harness.send(Message::WindowResized(WIDE));
    assert_eq!(harness.app.layout_mode, LayoutMode::Wide);
    assert_eq!(harness.app.open_drawer(), None);
    assert_eq!(harness.app.panes.drawer, None);
    assert!(
        has_id(&mut harness.ui(), ids::RAIL_CLEAR_ALL),
        "the rail is docked again"
    );
    Ok(())
}

#[test]
fn a_drawer_request_in_the_wide_layout_is_ignored() -> Result<(), AppError> {
    let mut harness = Harness::new(WIDE)?;
    harness.send(Message::Overlay(OverlayMessage::OpenDrawer(
        Drawer::Details,
    )));
    assert_eq!(harness.app.open_drawer(), None);
    assert_eq!(harness.app.panes.drawer, None);
    Ok(())
}
