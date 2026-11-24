//! Pane collapsing, hiding and resizing in the wide layout, and the mirrored order of the
//! panes in a right-to-left locale.

use iced::widget::Id;

use super::harness::{Harness, Ui, WIDE, ui_error};
use super::selectors::{bounds_of, has_id};
use crate::AppError;
use crate::app::i18n::Locale;
use crate::app::ids;
use crate::app::layout::{RAIL_MAX_WIDTH, RAIL_MIN_WIDTH};
use crate::app::message::{Message, PaneMessage};

/// Whether `messages` hold exactly the pane message `wanted`.
fn published(messages: &[Message], wanted: fn(&PaneMessage) -> bool) -> bool {
    messages
        .iter()
        .any(|message| matches!(message, Message::Pane(pane) if wanted(pane)))
}

#[test]
fn the_rail_collapses_to_a_strip_and_expands_again() -> Result<(), AppError> {
    let mut harness = Harness::new(WIDE)?;
    let mut ui = harness.ui();
    assert!(has_id(&mut ui, ids::RAIL_CLEAR_ALL));
    assert!(has_id(&mut ui, ids::RAIL_SPLITTER));
    let docked_grid = bounds_of(&mut ui, ids::GRID).map_err(ui_error)?;
    drop(ui);

    let messages = harness.drive(|ui| {
        let _button = ui.click(ids::RAIL_COLLAPSE)?;
        Ok(())
    })?;
    assert!(
        published(&messages, |pane| matches!(
            pane,
            PaneMessage::SetRailCollapsed(true)
        )),
        "{messages:?}"
    );
    assert!(harness.app.panes.rail_collapsed);

    let mut ui = harness.ui();
    assert!(
        !has_id(&mut ui, ids::RAIL_CLEAR_ALL),
        "the strip has no Clear all"
    );
    assert!(
        !has_id(&mut ui, ids::RAIL_SPLITTER),
        "the strip has no splitter"
    );
    assert!(
        has_id(&mut ui, ids::RAIL_COLLAPSE),
        "the expand button keeps the collapse id"
    );
    let wider = bounds_of(&mut ui, ids::GRID).map_err(ui_error)?;
    assert!(
        wider.width > docked_grid.width,
        "{} > {}",
        wider.width,
        docked_grid.width
    );
    drop(ui);

    let messages = harness.drive(|ui| {
        let _button = ui.click(ids::RAIL_COLLAPSE)?;
        Ok(())
    })?;
    assert!(
        published(&messages, |pane| matches!(
            pane,
            PaneMessage::SetRailCollapsed(false)
        )),
        "{messages:?}"
    );
    assert!(has_id(&mut harness.ui(), ids::RAIL_CLEAR_ALL));
    Ok(())
}

#[test]
fn the_detail_pane_hides_and_the_toolbar_offers_it_back() -> Result<(), AppError> {
    let mut harness = Harness::new(WIDE)?;
    let mut ui = harness.ui();
    assert!(has_id(&mut ui, ids::DETAIL_HIDE));
    assert!(!has_id(&mut ui, ids::SHOW_DETAILS_BUTTON));
    drop(ui);

    let messages = harness.drive(|ui| {
        let _button = ui.click(ids::DETAIL_HIDE)?;
        Ok(())
    })?;
    assert!(
        published(&messages, |pane| matches!(
            pane,
            PaneMessage::SetDetailHidden(true)
        )),
        "{messages:?}"
    );

    let mut ui = harness.ui();
    assert!(!has_id(&mut ui, ids::DETAIL_HIDE));
    assert!(!has_id(&mut ui, ids::DETAIL_SPLITTER));
    assert!(has_id(&mut ui, ids::SHOW_DETAILS_BUTTON));
    drop(ui);

    let messages = harness.drive(|ui| {
        let _button = ui.click(ids::SHOW_DETAILS_BUTTON)?;
        Ok(())
    })?;
    assert!(
        published(&messages, |pane| matches!(
            pane,
            PaneMessage::SetDetailHidden(false)
        )),
        "{messages:?}"
    );
    assert!(!harness.app.panes.detail_hidden);
    assert!(has_id(&mut harness.ui(), ids::DETAIL_HIDE));
    Ok(())
}

#[test]
fn rail_widths_are_clamped_and_non_finite_widths_are_ignored() -> Result<(), AppError> {
    let mut harness = Harness::new(WIDE)?;
    let before = harness.app.panes.rail_width;

    harness.send(Message::Pane(PaneMessage::ResizeRail(f32::NAN)));
    assert!((harness.app.panes.rail_width - before).abs() < f32::EPSILON);

    harness.send(Message::Pane(PaneMessage::ResizeRail(10_000.0)));
    assert!((harness.app.panes.rail_width - RAIL_MAX_WIDTH).abs() < f32::EPSILON);
    harness.send(Message::Pane(PaneMessage::ResizeRail(1.0)));
    assert!((harness.app.panes.rail_width - RAIL_MIN_WIDTH).abs() < f32::EPSILON);
    Ok(())
}

/// Horizontal centre of the widget reported under `id`.
fn centre_x(ui: &mut Ui<'_>, id: Id) -> Result<f32, AppError> {
    bounds_of(ui, id)
        .map(|bounds| bounds.center().x)
        .map_err(ui_error)
}

/// Horizontal centres of the rail splitter, the grid and the detail splitter.
fn pane_centres(harness: &Harness) -> Result<[f32; 3], AppError> {
    let mut ui = harness.ui();
    Ok([
        centre_x(&mut ui, ids::RAIL_SPLITTER)?,
        centre_x(&mut ui, ids::GRID)?,
        centre_x(&mut ui, ids::DETAIL_SPLITTER)?,
    ])
}

/// Horizontal centres of the search field and the About button.
fn header_centres(harness: &Harness) -> Result<[f32; 2], AppError> {
    let mut ui = harness.ui();
    Ok([
        centre_x(&mut ui, ids::SEARCH)?,
        centre_x(&mut ui, ids::ABOUT_BUTTON)?,
    ])
}

#[test]
fn a_right_to_left_locale_mirrors_the_panes_and_the_header() -> Result<(), AppError> {
    let ltr = Harness::new(WIDE)?;
    let rtl = Harness::with_locale(Locale::Ar, WIDE)?;
    assert!(!ltr.app.direction.is_rtl());
    assert!(rtl.app.direction.is_rtl());

    let [rail, grid, detail] = pane_centres(&ltr)?;
    assert!(
        rail < grid && grid < detail,
        "left to right: {rail} {grid} {detail}"
    );
    let [rail, grid, detail] = pane_centres(&rtl)?;
    assert!(
        detail < grid && grid < rail,
        "right to left: {detail} {grid} {rail}"
    );

    let [search, about] = header_centres(&ltr)?;
    assert!(
        search < about,
        "the About button ends the header: {search} {about}"
    );
    let [search, about] = header_centres(&rtl)?;
    assert!(
        about < search,
        "the About button ends the header: {about} {search}"
    );
    Ok(())
}
