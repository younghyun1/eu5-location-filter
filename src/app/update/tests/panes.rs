//! Pane geometry, drawers, surfaces, settings and keyboard tests.

use iced::Size;
use iced::time::Duration;

use super::fixture::{LAND, app};
use crate::AppError;
use crate::app::context::Platform;
use crate::app::i18n::Locale;
use crate::app::ids;
use crate::app::layout::{DETAIL_MIN_WIDTH, Direction, Drawer, LayoutMode, RAIL_MAX_WIDTH};
use crate::app::message::{
    GridMessage, KeyCommand, Message, OverlayMessage, PaneMessage, SelectionSource,
    SettingsMessage,
};
use crate::app::state::{App, FocusRegion, Surface};
use crate::app::theme::{AccentPreset, Mode, PALETTE_LIGHT, Palette};
use crate::app::update::focus::{Target, region_target};
use crate::app::update::update;

/// Sends one message and drops its task.
fn send(app: &mut App, message: Message) {
    let _task = update(app, message);
}

/// Switches the app to the narrow layout.
fn narrow(app: &mut App) {
    send(app, Message::WindowResized(Size::new(400.0, 800.0)));
}

#[test]
fn splitter_widths_are_clamped_and_non_finite_widths_dropped() -> Result<(), AppError> {
    let mut app = app(Platform::Desktop)?;
    send(&mut app, Message::Pane(PaneMessage::ResizeRail(10_000.0)));
    assert_eq!(app.panes.rail_width, RAIL_MAX_WIDTH);
    send(&mut app, Message::Pane(PaneMessage::ResizeRail(f32::NAN)));
    assert_eq!(app.panes.rail_width, RAIL_MAX_WIDTH);
    send(&mut app, Message::Pane(PaneMessage::ResizeDetail(0.0)));
    assert_eq!(app.panes.detail_width, DETAIL_MIN_WIDTH);
    send(&mut app, Message::Pane(PaneMessage::SetRailCollapsed(true)));
    assert!(app.panes.rail_collapsed);
    Ok(())
}

#[test]
fn drawers_open_only_in_the_narrow_layout_and_close_when_leaving_it() -> Result<(), AppError> {
    let mut app = app(Platform::Browser)?;
    send(&mut app, Message::Overlay(OverlayMessage::OpenDrawer(Drawer::Filters)));
    assert_eq!(app.panes.drawer, None);
    narrow(&mut app);
    assert_eq!(app.layout_mode, LayoutMode::Narrow);
    send(&mut app, Message::Overlay(OverlayMessage::OpenDrawer(Drawer::Filters)));
    assert_eq!(app.open_drawer(), Some(Drawer::Filters));
    assert_eq!(app.focus.return_to, Some(ids::FILTERS_BUTTON));
    send(&mut app, Message::WindowResized(Size::new(1200.0, 800.0)));
    assert_eq!(app.panes.drawer, None);
    assert_eq!(app.open_drawer(), None);
    Ok(())
}

#[test]
fn a_closing_drawer_stays_drawn_until_its_slide_ends() -> Result<(), AppError> {
    let mut app = app(Platform::Browser)?;
    narrow(&mut app);
    send(&mut app, Message::Overlay(OverlayMessage::OpenDrawer(Drawer::Details)));
    // Let the opening slide finish so closing starts from a settled drawer.
    let opened = app.appearance.now + Duration::from_secs(1);
    send(&mut app, Message::Frame(opened));
    assert_eq!(app.open_drawer(), Some(Drawer::Details));
    send(&mut app, Message::Key(KeyCommand::Escape));
    assert_eq!(app.open_drawer(), None);
    assert_eq!(app.panes.drawer, Some(Drawer::Details));
    let closed = app.appearance.now + Duration::from_secs(1);
    send(&mut app, Message::Frame(closed));
    assert_eq!(app.panes.drawer, None);
    Ok(())
}

#[test]
fn without_motion_a_drawer_closes_at_once() -> Result<(), AppError> {
    let mut app = app(Platform::Desktop)?;
    send(&mut app, Message::Settings(SettingsMessage::Motion(false)));
    assert!(app.appearance.reduced_motion);
    narrow(&mut app);
    send(&mut app, Message::Overlay(OverlayMessage::OpenDrawer(Drawer::Filters)));
    send(&mut app, Message::Overlay(OverlayMessage::CloseDrawer));
    assert_eq!(app.panes.drawer, None);
    Ok(())
}

#[test]
fn a_tap_opens_the_detail_drawer_and_a_key_does_not() -> Result<(), AppError> {
    let mut app = app(Platform::Browser)?;
    narrow(&mut app);
    let select = |source| Message::Grid(GridMessage::Select { id: LAND, source });
    send(&mut app, select(SelectionSource::Keyboard));
    assert_eq!(app.open_drawer(), None);
    send(&mut app, select(SelectionSource::Pointer));
    assert_eq!(app.open_drawer(), Some(Drawer::Details));
    Ok(())
}

#[test]
fn surfaces_toggle_and_desktop_menus_stay_closed_in_the_browser() -> Result<(), AppError> {
    let mut app = app(Platform::Desktop)?;
    send(&mut app, Message::Overlay(OverlayMessage::Toggle(Surface::ColumnMenu)));
    assert_eq!(app.overlay.surface, Some(Surface::ColumnMenu));
    assert_eq!(app.focus.return_to, Some(ids::COLUMNS_BUTTON));
    send(&mut app, Message::Overlay(OverlayMessage::Toggle(Surface::ColumnMenu)));
    assert_eq!(app.overlay.surface, None);
    assert_eq!(app.focus.return_to, None);
    let mut browser = super::fixture::app(Platform::Browser)?;
    send(&mut browser, Message::Overlay(OverlayMessage::Open(Surface::LanguageMenu)));
    assert_eq!(browser.overlay.surface, None);
    Ok(())
}

#[test]
fn escape_closes_the_topmost_surface_first() -> Result<(), AppError> {
    let mut app = app(Platform::Browser)?;
    narrow(&mut app);
    send(&mut app, Message::Overlay(OverlayMessage::OpenDrawer(Drawer::Filters)));
    send(&mut app, Message::Overlay(OverlayMessage::Open(Surface::About)));
    send(&mut app, Message::Key(KeyCommand::Escape));
    assert_eq!(app.overlay.surface, None);
    assert_eq!(app.panes.drawer, Some(Drawer::Filters));
    Ok(())
}

#[test]
fn settings_change_direction_and_palette_targets() -> Result<(), AppError> {
    let mut app = app(Platform::Desktop)?;
    send(&mut app, Message::Settings(SettingsMessage::Locale(Locale::Ar)));
    assert_eq!(app.direction, Direction::Rtl);
    send(&mut app, Message::Settings(SettingsMessage::Mode(Mode::Light)));
    assert_eq!(app.appearance.fade.target(), PALETTE_LIGHT);
    send(&mut app, Message::Settings(SettingsMessage::Accent(AccentPreset::Lagoon)));
    let expected = Palette::builtin(Mode::Light).with_accent(AccentPreset::Lagoon);
    assert_eq!(app.appearance.fade.target(), expected);
    let later = app.appearance.now + Duration::from_secs(1);
    send(&mut app, Message::Frame(later));
    assert_eq!(app.appearance.palette, expected);
    Ok(())
}

#[test]
fn region_targets_follow_the_layout() -> Result<(), AppError> {
    let mut app = app(Platform::Desktop)?;
    assert_eq!(region_target(&app, FocusRegion::Header), Some(Target::Search));
    assert_eq!(
        region_target(&app, FocusRegion::Rail),
        Some(Target::Widget(ids::RAIL_COLLAPSE))
    );
    send(&mut app, Message::Pane(PaneMessage::SetDetailHidden(true)));
    assert_eq!(region_target(&app, FocusRegion::Detail), None);
    send(&mut app, Message::Key(KeyCommand::PreviousRegion));
    assert_eq!(app.focus.region, FocusRegion::Grid);
    let mut browser = super::fixture::app(Platform::Browser)?;
    narrow(&mut browser);
    assert_eq!(region_target(&browser, FocusRegion::Rail), None);
    assert_eq!(
        region_target(&browser, FocusRegion::Header),
        Some(Target::Widget(ids::FILTERS_BUTTON))
    );
    Ok(())
}

#[test]
fn regions_do_not_cycle_while_a_surface_is_open() -> Result<(), AppError> {
    let mut app = app(Platform::Desktop)?;
    send(&mut app, Message::Overlay(OverlayMessage::Open(Surface::About)));
    send(&mut app, Message::Key(KeyCommand::NextRegion));
    assert_eq!(app.focus.region, FocusRegion::Header);
    send(&mut app, Message::Key(KeyCommand::FocusSearch));
    assert_eq!(app.overlay.surface, Some(Surface::About));
    Ok(())
}
