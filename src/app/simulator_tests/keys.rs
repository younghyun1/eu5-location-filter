//! Application keyboard commands. The simulator runs no subscriptions, so key presses never
//! become `Message::Key` there; the mapping from presses to commands is covered by the tests in
//! `subscription.rs`, and these tests send the commands the way the subscription would.
//! Widget-level keys (grid arrows, Space, Tab left alone) are tested in `table.rs`.

use super::harness::{Harness, NARROW, WIDE};
use crate::AppError;
use crate::app::layout::Drawer;
use crate::app::message::{KeyCommand, Message, OverlayMessage, PaneMessage};
use crate::app::state::{FocusRegion, Surface};

/// Sends `command`.
fn key(harness: &mut Harness, command: KeyCommand) {
    harness.send(Message::Key(command));
}

/// Regions reached by pressing F6 `presses` times from the header.
fn regions_forward(harness: &mut Harness, presses: usize) -> Vec<FocusRegion> {
    let mut seen = Vec::with_capacity(presses);
    for _ in 0..presses {
        key(harness, KeyCommand::NextRegion);
        seen.push(harness.app.focus.region);
    }
    seen
}

#[test]
fn escape_closes_a_menu() -> Result<(), AppError> {
    let mut harness = Harness::new(WIDE)?;
    harness.send(Message::Overlay(OverlayMessage::Open(Surface::ColumnMenu)));
    assert_eq!(harness.app.overlay.surface, Some(Surface::ColumnMenu));
    key(&mut harness, KeyCommand::Escape);
    assert_eq!(harness.app.overlay.surface, None);
    key(&mut harness, KeyCommand::Escape);
    assert_eq!(
        harness.app.overlay.surface, None,
        "Escape with nothing open does nothing"
    );
    Ok(())
}

#[test]
fn escape_closes_the_dialog_before_the_drawer_beneath_it() -> Result<(), AppError> {
    let mut harness = Harness::new(NARROW)?;
    harness.send(Message::Overlay(OverlayMessage::OpenDrawer(
        Drawer::Filters,
    )));
    harness.settle();
    harness.send(Message::Overlay(OverlayMessage::Open(Surface::About)));
    assert_eq!(harness.app.overlay.surface, Some(Surface::About));
    assert_eq!(harness.app.open_drawer(), Some(Drawer::Filters));

    key(&mut harness, KeyCommand::Escape);
    assert_eq!(harness.app.overlay.surface, None);
    assert_eq!(
        harness.app.open_drawer(),
        Some(Drawer::Filters),
        "the drawer is still open"
    );

    key(&mut harness, KeyCommand::Escape);
    assert_eq!(harness.app.open_drawer(), None);
    Ok(())
}

#[test]
fn f6_cycles_the_regions_that_are_shown() -> Result<(), AppError> {
    let mut harness = Harness::new(WIDE)?;
    assert_eq!(harness.app.focus.region, FocusRegion::Header);
    let all = [
        FocusRegion::Rail,
        FocusRegion::Grid,
        FocusRegion::Detail,
        FocusRegion::Header,
    ];
    assert_eq!(regions_forward(&mut harness, 4), all);

    key(&mut harness, KeyCommand::PreviousRegion);
    assert_eq!(harness.app.focus.region, FocusRegion::Detail);
    key(&mut harness, KeyCommand::PreviousRegion);
    assert_eq!(harness.app.focus.region, FocusRegion::Grid);
    Ok(())
}

#[test]
fn f6_skips_a_hidden_detail_pane() -> Result<(), AppError> {
    let mut harness = Harness::new(WIDE)?;
    harness.send(Message::Pane(PaneMessage::SetDetailHidden(true)));
    let expected = [FocusRegion::Rail, FocusRegion::Grid, FocusRegion::Header];
    assert_eq!(regions_forward(&mut harness, 3), expected);
    Ok(())
}

#[test]
fn f6_skips_the_rail_and_the_detail_pane_in_the_narrow_layout() -> Result<(), AppError> {
    let mut harness = Harness::new(NARROW)?;
    let expected = [FocusRegion::Grid, FocusRegion::Header, FocusRegion::Grid];
    assert_eq!(regions_forward(&mut harness, 3), expected);
    Ok(())
}

#[test]
fn f6_stays_inside_an_open_surface() -> Result<(), AppError> {
    let mut harness = Harness::new(WIDE)?;
    harness.send(Message::Overlay(OverlayMessage::Open(Surface::ColumnMenu)));
    key(&mut harness, KeyCommand::NextRegion);
    assert_eq!(harness.app.focus.region, FocusRegion::Header);
    Ok(())
}

#[test]
fn the_search_shortcut_moves_the_focus_region_to_the_header() -> Result<(), AppError> {
    let mut harness = Harness::new(WIDE)?;
    regions_forward(&mut harness, 2);
    assert_eq!(harness.app.focus.region, FocusRegion::Grid);
    key(&mut harness, KeyCommand::FocusSearch);
    assert_eq!(harness.app.focus.region, FocusRegion::Header);
    Ok(())
}

#[test]
fn the_search_shortcut_closes_a_menu_but_not_a_dialog() -> Result<(), AppError> {
    let mut harness = Harness::new(WIDE)?;
    harness.send(Message::Overlay(OverlayMessage::Open(Surface::ColumnMenu)));
    key(&mut harness, KeyCommand::FocusSearch);
    assert_eq!(
        harness.app.overlay.surface, None,
        "focus leaves the menu, so it closes"
    );

    harness.send(Message::Overlay(OverlayMessage::Open(Surface::About)));
    key(&mut harness, KeyCommand::FocusSearch);
    assert_eq!(
        harness.app.overlay.surface,
        Some(Surface::About),
        "the dialog covers the search"
    );
    Ok(())
}

#[test]
fn the_search_shortcut_is_ignored_while_a_drawer_covers_the_header() -> Result<(), AppError> {
    let mut harness = Harness::new(NARROW)?;
    harness.send(Message::Overlay(OverlayMessage::OpenDrawer(
        Drawer::Details,
    )));
    harness.settle();
    key(&mut harness, KeyCommand::NextRegion);
    assert_eq!(
        harness.app.focus.region,
        FocusRegion::Header,
        "a drawer holds focus too"
    );
    key(&mut harness, KeyCommand::FocusSearch);
    assert_eq!(harness.app.open_drawer(), Some(Drawer::Details));
    Ok(())
}
