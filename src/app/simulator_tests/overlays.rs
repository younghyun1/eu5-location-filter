//! Loading and error veils, the About dialog and the desktop-only header menus.

use std::sync::Arc;

use iced::Point;
use iced_test::simulator::click;

use super::fixture;
use super::harness::{Harness, WIDE};
use super::selectors::has_id;
use crate::AppError;
use crate::app::context::Platform;
use crate::app::i18n::{Locale, Text, t};
use crate::app::ids;
use crate::app::message::{LoadMessage, Message, OverlayMessage};
use crate::app::state::{App, LaunchPaths, Phase, Surface};
use crate::app::theme::PALETTE_DARK;

/// An application that has not loaded anything yet.
fn unloaded(platform: Platform) -> Harness {
    let app = App::new(platform, LaunchPaths::default(), Locale::EnUs, PALETTE_DARK);
    Harness::from_app(app, WIDE)
}

/// An application whose first load failed with `error`.
fn failed(platform: Platform, error: AppError) -> Harness {
    let mut harness = unloaded(platform);
    let generation = harness.app.load_generation;
    harness.send(Message::Load(LoadMessage::Finished {
        generation,
        result: Err(Arc::new(error)),
    }));
    harness
}

fn missing_file() -> AppError {
    AppError::MissingFile("common/location_templates.txt".into())
}

#[test]
fn the_error_card_offers_retry_and_rebuild_on_the_desktop() -> Result<(), AppError> {
    let harness = failed(Platform::Desktop, missing_file());
    assert!(matches!(harness.app.phase, Phase::Failed(_)));

    let mut ui = harness.ui();
    assert!(ui.find(t(Locale::EnUs, Text::ErrorTitle)).is_ok());
    assert!(ui.find(t(Locale::EnUs, Text::ErrorHintRetryable)).is_ok());
    assert!(
        ui.find(missing_file().to_string()).is_ok(),
        "the technical detail is shown"
    );
    assert!(has_id(&mut ui, ids::INSTALL_PATH));
    drop(ui);

    // Retry and Rebuild start a loader thread when applied, so only what they publish is checked.
    let retry = harness.interact(|ui| {
        let _button = ui.click(t(Locale::EnUs, Text::ErrorRetry))?;
        Ok(())
    })?;
    assert!(
        matches!(retry.as_slice(), [Message::Load(LoadMessage::Retry)]),
        "{retry:?}"
    );
    let rebuild = harness.interact(|ui| {
        let _button = ui.click(t(Locale::EnUs, Text::ErrorRebuild))?;
        Ok(())
    })?;
    assert!(
        matches!(rebuild.as_slice(), [Message::Load(LoadMessage::Rebuild)]),
        "{rebuild:?}"
    );
    Ok(())
}

#[test]
fn the_installation_path_field_keeps_what_is_typed() -> Result<(), AppError> {
    let mut harness = failed(Platform::Desktop, missing_file());
    harness.drive(|ui| {
        let _field = ui.click(ids::INSTALL_PATH)?;
        let _status = ui.typewrite("games");
        Ok(())
    })?;
    assert_eq!(harness.app.fields.install_path, "games");
    assert!(
        matches!(harness.app.phase, Phase::Failed(_)),
        "typing does not restart the load"
    );
    Ok(())
}

#[test]
fn the_browser_error_card_asks_for_a_reload_instead() -> Result<(), AppError> {
    let harness = failed(Platform::Browser, AppError::Fetch("status 503".to_owned()));
    let mut ui = harness.ui();
    assert!(ui.find(t(Locale::EnUs, Text::ErrorTitle)).is_ok());
    assert!(ui.find(t(Locale::EnUs, Text::ErrorHintWeb)).is_ok());
    assert!(
        ui.find(t(Locale::EnUs, Text::ErrorRetry)).is_err(),
        "no Retry in the browser"
    );
    assert!(
        ui.find(t(Locale::EnUs, Text::ErrorRebuild)).is_err(),
        "no Rebuild in the browser"
    );
    assert!(!has_id(&mut ui, ids::INSTALL_PATH));
    Ok(())
}

#[test]
fn the_error_veil_blocks_the_interface_beneath_it() -> Result<(), AppError> {
    let harness = failed(Platform::Desktop, missing_file());
    let messages = harness.interact(|ui| {
        let _button = ui.click(ids::COLUMNS_BUTTON)?;
        Ok(())
    })?;
    assert!(messages.is_empty(), "{messages:?}");
    Ok(())
}

#[test]
fn the_loading_card_follows_progress_and_blocks_the_interface() -> Result<(), AppError> {
    let mut harness = unloaded(Platform::Desktop);
    assert!(matches!(harness.app.phase, Phase::AwaitingPaint));
    assert!(
        harness
            .ui()
            .find(t(Locale::EnUs, Text::LoadStarting))
            .is_ok()
    );

    let generation = harness.app.load_generation;
    harness.send(Message::Load(LoadMessage::Progress {
        generation,
        stage: Text::LoadEmbedded,
        fraction: 0.5,
    }));
    assert!(matches!(harness.app.phase, Phase::Loading(_)));
    assert!(
        harness
            .ui()
            .find(t(Locale::EnUs, Text::LoadEmbedded))
            .is_ok()
    );

    let messages = harness.interact(|ui| {
        let _button = ui.click(ids::COLUMNS_BUTTON)?;
        Ok(())
    })?;
    assert!(messages.is_empty(), "{messages:?}");
    Ok(())
}

#[test]
fn the_about_dialog_closes_from_its_button_and_from_the_scrim() -> Result<(), AppError> {
    let mut harness = Harness::new(WIDE)?;
    let close = t(Locale::EnUs, Text::AboutClose);
    assert!(
        harness.ui().find(close).is_err(),
        "the dialog starts closed"
    );

    harness.drive(|ui| {
        let _button = ui.click(ids::ABOUT_BUTTON)?;
        Ok(())
    })?;
    assert_eq!(harness.app.overlay.surface, Some(Surface::About));
    assert!(harness.ui().find(close).is_ok());

    harness.drive(|ui| {
        let _button = ui.click(close)?;
        Ok(())
    })?;
    assert_eq!(harness.app.overlay.surface, None);

    harness.send(Message::Overlay(OverlayMessage::Open(Surface::About)));
    let messages = harness.interact(|ui| {
        // The dialog is centred and narrower than the window, so its edge is scrim.
        ui.point_at(Point::new(10.0, WIDE.height / 2.0));
        let _statuses = ui.simulate(click());
        Ok(())
    })?;
    harness.apply(messages);
    assert_eq!(harness.app.overlay.surface, None);
    Ok(())
}

#[test]
fn only_the_desktop_header_has_the_language_and_appearance_menus() -> Result<(), AppError> {
    let desktop = Harness::new(WIDE)?;
    let mut ui = desktop.ui();
    assert!(has_id(&mut ui, ids::LANGUAGE_BUTTON));
    assert!(has_id(&mut ui, ids::APPEARANCE_BUTTON));
    drop(ui);

    let app = App::with_data(
        Platform::Browser,
        Locale::EnUs,
        PALETTE_DARK,
        fixture::loaded()?,
    );
    let browser = Harness::from_app(app, WIDE);
    let mut ui = browser.ui();
    assert!(!has_id(&mut ui, ids::LANGUAGE_BUTTON));
    assert!(!has_id(&mut ui, ids::APPEARANCE_BUTTON));
    assert!(has_id(&mut ui, ids::ABOUT_BUTTON));
    Ok(())
}

#[test]
fn the_language_menu_opens_from_its_button() -> Result<(), AppError> {
    let mut harness = Harness::new(WIDE)?;
    harness.drive(|ui| {
        let _button = ui.click(ids::LANGUAGE_BUTTON)?;
        Ok(())
    })?;
    assert_eq!(harness.app.overlay.surface, Some(Surface::LanguageMenu));
    Ok(())
}
