//! Host snapshot, view-state queueing, reporting and search overlay tests.

use iced::Rectangle;

use super::fixture::app;
use crate::AppError;
use crate::app::context::Platform;
use crate::app::i18n::Locale;
use crate::app::message::{HostMessage, HostSnapshot, Message};
use crate::app::state::{App, LaunchPaths, Surface};
use crate::app::theme::{PALETTE_DARK, PALETTE_LIGHT};
use crate::app::update::host::{canonical_view_state, search_overlay_bounds};
use crate::app::update::update;

/// Browser application before any data.
fn booting() -> App {
    App::new(
        Platform::Browser,
        LaunchPaths::default(),
        Locale::EnUs,
        PALETTE_DARK,
    )
}

/// A snapshot as the first host message produces it.
fn first_message(view_state: Option<&str>) -> HostSnapshot {
    HostSnapshot {
        palette: Some(PALETTE_LIGHT),
        locale: Some(Locale::EnUs),
        reduced_motion: Some(true),
        view_state: view_state.map(str::to_owned),
        connected: true,
        search_text: None,
    }
}

/// Sends one host snapshot and drops its task.
fn send(app: &mut App, snapshot: HostSnapshot) {
    let _task = update(app, Message::Host(HostMessage::Update(snapshot)));
}

#[test]
fn a_view_state_waits_for_data_and_blocks_reporting() {
    let mut app = booting();
    send(&mut app, first_message(Some("s1~q.river")));
    assert!(app.host.connected);
    assert_eq!(app.host.pending_view_state.as_deref(), Some("s1~q.river"));
    assert_eq!(app.appearance.fade.target(), PALETTE_LIGHT);
    // Reduced motion applies the palette without a fade.
    assert_eq!(app.appearance.palette, PALETTE_LIGHT);
    assert_eq!(canonical_view_state(&app), None);
    assert_eq!(app.host.last_reported, None);
}

#[test]
fn reporting_starts_when_connected_and_ready() -> Result<(), AppError> {
    let mut app = app(Platform::Browser)?;
    assert_eq!(canonical_view_state(&app), None);
    send(&mut app, first_message(None));
    let reported = app.host.last_reported.clone();
    assert!(
        reported
            .as_deref()
            .is_some_and(|state| state.starts_with("s1"))
    );
    assert_eq!(canonical_view_state(&app), reported);
    Ok(())
}

#[test]
fn a_pending_view_state_is_consumed_once_data_is_ready() -> Result<(), AppError> {
    let mut app = app(Platform::Browser)?;
    send(&mut app, first_message(Some("s1")));
    assert_eq!(app.host.pending_view_state, None);
    assert!(app.filters.search.is_empty());
    Ok(())
}

#[test]
fn host_search_text_applies_like_typed_search() -> Result<(), AppError> {
    let mut app = app(Platform::Browser)?;
    send(
        &mut app,
        HostSnapshot {
            search_text: Some("stock".to_owned()),
            ..HostSnapshot::default()
        },
    );
    assert_eq!(app.filters.search, "stock");
    assert!(!app.host.connected);
    Ok(())
}

#[test]
fn the_search_overlay_hides_under_modal_surfaces() -> Result<(), AppError> {
    let mut app = app(Platform::Browser)?;
    assert_eq!(search_overlay_bounds(&app), None);
    let bounds = Rectangle::new(iced::Point::new(10.0, 8.0), iced::Size::new(300.0, 36.0));
    let _task = update(&mut app, Message::Host(HostMessage::SearchAnchor(bounds)));
    assert_eq!(search_overlay_bounds(&app), Some(bounds));
    let broken = Rectangle::new(iced::Point::new(f32::NAN, 8.0), iced::Size::new(1.0, 1.0));
    let _task = update(&mut app, Message::Host(HostMessage::SearchAnchor(broken)));
    assert_eq!(app.host.search_anchor, Some(bounds));
    app.overlay.surface = Some(Surface::About);
    assert_eq!(search_overlay_bounds(&app), None);
    app.overlay.surface = Some(Surface::ColumnMenu);
    assert_eq!(search_overlay_bounds(&app), Some(bounds));
    let mut loading = booting();
    loading.host.search_anchor = Some(bounds);
    assert_eq!(search_overlay_bounds(&loading), None);
    Ok(())
}
