//! Global search and Clear all through the real header and rail controls.

use super::fixture::{BUILD_ID, LAND_BY_NAME, LAND_COUNT};
use super::harness::{Harness, WIDE};
use super::selectors::{grid_shows, has_id};
use crate::AppError;
use crate::app::i18n::{Locale, Text, format, t};
use crate::app::ids;
use crate::app::message::{FilterMessage, GridMessage, Message, SelectionSource};
use crate::model::EU5_GAME_VERSION;

#[test]
fn typing_in_the_search_field_narrows_the_results() -> Result<(), AppError> {
    let mut harness = Harness::new(WIDE)?;
    assert_eq!(harness.results()?, LAND_BY_NAME.as_slice());

    let messages = harness.drive(|ui| {
        let _field = ui.click(ids::SEARCH)?;
        let _status = ui.typewrite("vik");
        Ok(())
    })?;

    assert!(
        messages
            .iter()
            .any(|message| matches!(message, Message::Filter(FilterMessage::Search(text)) if text == "vik")),
        "{messages:?}"
    );
    assert_eq!(harness.app.filters.search, "vik");
    assert_eq!(harness.results()?.len(), 1);

    let mut ui = harness.ui();
    assert!(ui.find(harness.count_text(1)).is_ok());
    assert!(grid_shows(&mut ui, "Brevik"));
    assert!(!grid_shows(&mut ui, "Aldoria"));
    Ok(())
}

#[test]
fn the_header_reports_the_build_and_the_unfiltered_count() -> Result<(), AppError> {
    let harness = Harness::new(WIDE)?;
    let mut ui = harness.ui();
    assert!(ui.find(harness.count_text(LAND_COUNT)).is_ok());
    let build_id = BUILD_ID.to_string();
    let build = format(
        t(Locale::EnUs, Text::HeaderBuild),
        &[("version", EU5_GAME_VERSION), ("build", build_id.as_str())],
    );
    assert!(ui.find(build).is_ok());
    Ok(())
}

#[test]
fn a_search_without_matches_empties_the_grid_and_the_selection() -> Result<(), AppError> {
    let mut harness = Harness::new(WIDE)?;
    harness.send(Message::Grid(GridMessage::Select {
        id: LAND_BY_NAME[0],
        source: SelectionSource::Pointer,
    }));
    assert_eq!(harness.app.selection, Some(LAND_BY_NAME[0]));

    harness.send(Message::Filter(FilterMessage::Search("zzz".to_owned())));

    assert!(harness.results()?.is_empty());
    assert_eq!(harness.app.selection, None);
    assert!(harness.app.detail.is_none());
    let mut ui = harness.ui();
    assert!(ui.find(harness.count_text(0)).is_ok());
    assert!(has_id(&mut ui, ids::GRID), "the grid stays on screen when empty");
    Ok(())
}

#[test]
fn clear_all_restores_the_default_view() -> Result<(), AppError> {
    let mut harness = Harness::new(WIDE)?;
    harness.send(Message::Filter(FilterMessage::Search("vik".to_owned())));
    assert_eq!(harness.results()?.len(), 1);

    let messages = harness.drive(|ui| {
        let _button = ui.click(ids::RAIL_CLEAR_ALL)?;
        Ok(())
    })?;

    assert!(
        messages
            .iter()
            .any(|message| matches!(message, Message::Filter(FilterMessage::ClearAll))),
        "{messages:?}"
    );
    assert!(harness.app.filters.search.is_empty());
    assert_eq!(harness.results()?, LAND_BY_NAME.as_slice());
    Ok(())
}
