//! Data load progress and results, desktop retry and rebuild, and language and font packs.

use std::path::PathBuf;
use std::sync::Arc;

use iced::Task;

use super::{host, packs};
use crate::AppError;
use crate::app::handoff::Handoff;
use crate::app::i18n::Text;
use crate::app::load::{self, LoadedData};
use crate::app::message::{LoadMessage, Message};
use crate::app::state::{App, LaunchPaths, LoadFailure, LoadProgress, Phase};

/// Handles loading messages. Stale generations are ignored. A finished load installs the data,
/// builds options, applies a pending view state, and requests the language and font packs of
/// the current locale. Pack failures set a notice and record the failure kind.
pub(super) fn load(app: &mut App, message: LoadMessage) -> Task<Message> {
    match message {
        LoadMessage::Painted => {
            // The browser reports every frame until the phase changes, so only the first
            // painted frame starts the decode.
            if matches!(app.phase, Phase::AwaitingPaint) {
                let launch = app.launch.clone();
                begin(app, &launch, false)
            } else {
                Task::none()
            }
        }
        LoadMessage::Progress {
            generation,
            stage,
            fraction,
        } => {
            // Desktop boot starts generation 0 while still awaiting paint, so progress is
            // accepted by generation and by any phase that is not settled yet.
            let open = matches!(app.phase, Phase::AwaitingPaint | Phase::Loading(_));
            if generation == app.load_generation && open {
                app.phase = Phase::Loading(LoadProgress {
                    stage,
                    fraction: clamp_fraction(fraction),
                });
            }
            Task::none()
        }
        LoadMessage::Finished { generation, result } => finished(app, generation, result),
        LoadMessage::InstallPath(text) => {
            app.fields.install_path = text;
            Task::none()
        }
        LoadMessage::Retry => restart(app, false),
        LoadMessage::Rebuild => restart(app, true),
        LoadMessage::LanguagePackFetched { language, result } => {
            packs::fetched(app, language, result)
        }
        LoadMessage::LanguagePackValidated { language, result } => {
            packs::validated(app, language, result)
        }
        LoadMessage::FontPackLoaded { pack, result } => packs::font_loaded(app, pack, result),
    }
}

/// Starts a load with a new generation, so anything the previous load still reports is
/// ignored.
fn begin(app: &mut App, launch: &LaunchPaths, force: bool) -> Task<Message> {
    app.load_generation = app.load_generation.wrapping_add(1);
    app.phase = Phase::Loading(LoadProgress {
        stage: if force {
            Text::LoadRebuild
        } else {
            Text::LoadStarting
        },
        fraction: 0.0,
    });
    load::start(launch, app.load_generation, force)
}

/// Desktop Retry and Rebuild from the error card. A typed installation directory overrides
/// the command-line one for this attempt only; an empty field falls back to it, as before.
fn restart(app: &mut App, force: bool) -> Task<Message> {
    if !app.platform.has_desktop_controls() || !matches!(app.phase, Phase::Failed(_)) {
        return Task::none();
    }
    let mut launch = app.launch.clone();
    let typed = app.fields.install_path.trim();
    if !typed.is_empty() {
        launch.game_dir = Some(PathBuf::from(typed));
    }
    begin(app, &launch, force)
}

/// Installs a finished load of the current generation or shows its failure.
fn finished(
    app: &mut App,
    generation: u64,
    result: Result<Handoff<LoadedData>, Arc<AppError>>,
) -> Task<Message> {
    if generation != app.load_generation {
        return Task::none();
    }
    match result {
        Ok(handoff) => match handoff.take() {
            Some(data) => {
                app.install_data(data);
                let view = host::apply_pending_view_state(app);
                let packs = packs::request_packs(app);
                Task::batch([view, packs])
            }
            // An earlier delivery of the same message already installed the data.
            None => Task::none(),
        },
        Err(error) => {
            if !matches!(app.phase, Phase::Ready) {
                app.phase = Phase::Failed(LoadFailure { error });
            }
            Task::none()
        }
    }
}

/// Progress for the bar: finite and within `0.0..=1.0`.
fn clamp_fraction(fraction: f32) -> f32 {
    if fraction.is_finite() {
        fraction.clamp(0.0, 1.0)
    } else {
        0.0
    }
}
