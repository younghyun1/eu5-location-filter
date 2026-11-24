//! Load generation, install, retry guard and pack result tests.

use std::sync::Arc;

use super::fixture::{app, loaded};
use crate::AppError;
use crate::app::context::Platform;
use crate::app::handoff::Handoff;
use crate::app::i18n::{FontPack, Locale, Text};
use crate::app::message::{LoadMessage, Message};
use crate::app::state::{App, LaunchPaths, Notice, PackFailure, Phase};
use crate::app::theme::PALETTE_DARK;
use crate::app::update::packs::{failure_kind, request_packs};
use crate::app::update::update;
use crate::lang_pack::GameLanguage;

/// Sends one load message and drops its task.
fn send(app: &mut App, message: LoadMessage) {
    let _task = update(app, Message::Load(message));
}

/// Desktop application before any data, as boot leaves it.
fn booting() -> App {
    App::new(
        Platform::Desktop,
        LaunchPaths::default(),
        Locale::EnUs,
        PALETTE_DARK,
    )
}

#[test]
fn progress_of_another_generation_is_ignored() -> Result<(), AppError> {
    let mut app = booting();
    send(
        &mut app,
        LoadMessage::Progress {
            generation: 5,
            stage: Text::LoadEmbedded,
            fraction: 0.5,
        },
    );
    assert!(matches!(app.phase, Phase::AwaitingPaint));
    send(
        &mut app,
        LoadMessage::Progress {
            generation: 0,
            stage: Text::LoadEmbedded,
            fraction: f32::NAN,
        },
    );
    let Phase::Loading(progress) = app.phase else {
        return Err(AppError::InvalidData(
            "progress of the current generation must show".to_owned(),
        ));
    };
    assert_eq!(progress.stage, Text::LoadEmbedded);
    assert_eq!(progress.fraction, 0.0);
    Ok(())
}

#[test]
fn a_finished_load_installs_once() -> Result<(), AppError> {
    let mut app = booting();
    let handoff = Handoff::new(loaded()?);
    let finished = |handoff: &Handoff<_>| LoadMessage::Finished {
        generation: 0,
        result: Ok(handoff.clone()),
    };
    send(&mut app, finished(&handoff));
    assert!(matches!(app.phase, Phase::Ready));
    assert!(app.data.is_some());
    app.filters.search = "kept".to_owned();
    send(&mut app, finished(&handoff));
    assert_eq!(app.filters.search, "kept");
    Ok(())
}

#[test]
fn failures_show_and_stale_results_do_not() {
    let mut app = booting();
    send(
        &mut app,
        LoadMessage::Finished {
            generation: 3,
            result: Err(Arc::new(AppError::Worker)),
        },
    );
    assert!(matches!(app.phase, Phase::AwaitingPaint));
    send(
        &mut app,
        LoadMessage::Finished {
            generation: 0,
            result: Err(Arc::new(AppError::Worker)),
        },
    );
    assert!(matches!(app.phase, Phase::Failed(_)));
}

#[test]
fn retry_is_ignored_unless_the_load_failed() -> Result<(), AppError> {
    let mut app = app(Platform::Desktop)?;
    let generation = app.load_generation;
    send(&mut app, LoadMessage::Retry);
    send(&mut app, LoadMessage::Rebuild);
    assert_eq!(app.load_generation, generation);
    assert!(matches!(app.phase, Phase::Ready));
    Ok(())
}

#[test]
fn pack_failures_are_classified() {
    let mismatch = AppError::PackMismatch("other build".to_owned());
    let fetch = AppError::Fetch("offline".to_owned());
    let corrupt = AppError::InvalidData("bad pack".to_owned());
    assert_eq!(failure_kind(&mismatch), PackFailure::Mismatch);
    assert_eq!(failure_kind(&fetch), PackFailure::Retryable);
    assert_eq!(failure_kind(&corrupt), PackFailure::Permanent);
}

#[test]
fn a_failed_pack_shows_a_notice_and_is_not_requested_again() -> Result<(), AppError> {
    let mut app = app(Platform::Browser)?;
    // Set directly: a locale message would start a real pack load.
    app.locale = Locale::KoKr;
    send(
        &mut app,
        LoadMessage::LanguagePackFetched {
            language: GameLanguage::Korean,
            result: Err(Arc::new(AppError::PackMismatch("build 6".to_owned()))),
        },
    );
    assert_eq!(app.overlay.notice, Some(Notice::LanguagePackMismatch));
    assert_eq!(
        app.packs.language_failure(GameLanguage::Korean),
        Some(PackFailure::Mismatch)
    );
    let _task = request_packs(&mut app);
    assert_eq!(app.packs.pending_language, None);
    Ok(())
}

#[test]
fn results_for_a_language_no_longer_wanted_are_dropped() -> Result<(), AppError> {
    let mut app = app(Platform::Browser)?;
    app.locale = Locale::KoKr;
    send(
        &mut app,
        LoadMessage::LanguagePackFetched {
            language: GameLanguage::French,
            result: Err(Arc::new(AppError::Fetch("offline".to_owned()))),
        },
    );
    assert_eq!(app.overlay.notice, None);
    assert_eq!(app.packs.language_failure(GameLanguage::French), None);
    Ok(())
}

#[test]
fn font_pack_results_update_the_fixed_bookkeeping() -> Result<(), AppError> {
    let mut app = app(Platform::Browser)?;
    let pack = FontPack::PlexSansKr;
    if let Some(slot) = app.packs.fonts_pending.get_mut(pack.index()) {
        *slot = true;
    }
    send(
        &mut app,
        LoadMessage::FontPackLoaded {
            pack,
            result: Err(Arc::new(AppError::Fetch("offline".to_owned()))),
        },
    );
    assert_eq!(app.overlay.notice, Some(Notice::FontPack));
    assert_eq!(app.packs.fonts_pending.get(pack.index()), Some(&false));
    send(
        &mut app,
        LoadMessage::FontPackLoaded {
            pack,
            result: Ok(()),
        },
    );
    assert_eq!(app.packs.fonts_loaded.get(pack.index()), Some(&true));
    Ok(())
}
