//! Background data-file loading and import orchestration.

use std::path::PathBuf;

use slint::{ComponentHandle, SharedString, Weak};

use super::{AppWindow, state};
use crate::import::{ImportProgress, import_game};
use crate::{AppError, steam, storage};

pub(super) fn install_action_callbacks(
    app: &AppWindow,
    data_file: PathBuf,
    default_game_dir: Option<PathBuf>,
) {
    let weak = app.as_weak();
    let retry_data = data_file.clone();
    let retry_default = default_game_dir.clone();
    app.on_retry(move |path| {
        let game_dir = selected_game_dir(path.as_str(), retry_default.as_ref());
        start_load(weak.clone(), retry_data.clone(), game_dir, false);
    });
    let weak = app.as_weak();
    app.on_rebuild(move |path| {
        let game_dir = selected_game_dir(path.as_str(), default_game_dir.as_ref());
        start_load(weak.clone(), data_file.clone(), game_dir, true);
    });
}

pub(super) fn start_load(
    weak: Weak<AppWindow>,
    data_file: PathBuf,
    game_dir: Option<PathBuf>,
    force: bool,
) {
    if let Some(app) = weak.upgrade() {
        app.set_loading(true);
        app.set_error_text(SharedString::new());
        app.set_progress_stage(
            if force {
                "Preparing rebuild"
            } else {
                "Loading data"
            }
            .into(),
        );
        app.set_progress_value(0.0);
    }
    std::thread::spawn(move || {
        let result = load_or_import(&weak, &data_file, game_dir.as_deref(), force).map(|dataset| {
            send_progress(
                &weak,
                ImportProgress {
                    stage: "Preparing search and sort indexes",
                    current: 1,
                    total: 1,
                },
            );
            state::prepare(dataset)
        });
        let completion = weak.clone();
        let dispatch = slint::invoke_from_event_loop(move || {
            let Some(app) = completion.upgrade() else {
                return;
            };
            match result {
                Ok(dataset) => state::configure(&app, dataset),
                Err(error) => {
                    app.set_loading(false);
                    app.set_error_text(
                        format!("{error}\n\nRetryable: {}", error.is_retryable()).into(),
                    );
                }
            }
        });
        if let Err(error) = dispatch {
            eprintln!("failed to deliver background result: {error}");
        }
    });
}

fn load_or_import(
    weak: &Weak<AppWindow>,
    data_file: &std::path::Path,
    game_dir: Option<&std::path::Path>,
    force: bool,
) -> Result<crate::model::Dataset, AppError> {
    if data_file.is_file() && !force {
        send_progress(
            weak,
            ImportProgress {
                stage: "Validating data file",
                current: 1,
                total: 1,
            },
        );
        return storage::load_dataset(data_file);
    }
    let installation = steam::discover(game_dir)?;
    let stored = import_game(&installation, |progress| send_progress(weak, progress))?;
    storage::write_dataset(data_file, &stored, force)?;
    storage::load_dataset(data_file)
}

fn send_progress(weak: &Weak<AppWindow>, progress: ImportProgress) {
    let update = weak.clone();
    let _ = slint::invoke_from_event_loop(move || {
        let Some(app) = update.upgrade() else { return };
        app.set_progress_stage(progress.stage.into());
        let value = if progress.total == 0 {
            0.0
        } else {
            progress.current as f32 / progress.total as f32
        };
        app.set_progress_value(value);
    });
}

fn selected_game_dir(value: &str, fallback: Option<&PathBuf>) -> Option<PathBuf> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback.cloned()
    } else {
        Some(PathBuf::from(trimmed))
    }
}
