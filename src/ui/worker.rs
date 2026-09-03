//! Background embedded loading and optional external rebuild orchestration.

use std::path::{Path, PathBuf};

use slint::{ComponentHandle, SharedString, Weak};

use super::{AppWindow, state};
use crate::filter::{FilterEngine, StoredFilterIndex};
use crate::import::{ImportProgress, import_game};
use crate::model::Dataset;
use crate::{AppError, embedded, index_storage, steam, storage};

pub(super) fn install_action_callbacks(
    app: &AppWindow,
    data_file: Option<PathBuf>,
    index_file: Option<PathBuf>,
    default_game_dir: Option<PathBuf>,
) {
    let weak = app.as_weak();
    let retry_data = data_file.clone();
    let retry_index = index_file.clone();
    let retry_default = default_game_dir.clone();
    app.on_retry(move |path| {
        let game_dir = selected_game_dir(path.as_str(), retry_default.as_ref());
        start_load(
            weak.clone(),
            retry_data.clone(),
            retry_index.clone(),
            game_dir,
            false,
        );
    });
    let weak = app.as_weak();
    app.on_rebuild(move |path| {
        let game_dir = selected_game_dir(path.as_str(), default_game_dir.as_ref());
        start_load(
            weak.clone(),
            data_file.clone(),
            index_file.clone(),
            game_dir,
            true,
        );
    });
}

pub(super) fn start_load(
    weak: Weak<AppWindow>,
    data_file: Option<PathBuf>,
    index_file: Option<PathBuf>,
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
                "Loading embedded data"
            }
            .into(),
        );
        app.set_progress_value(0.0);
    }
    std::thread::spawn(move || {
        let loaded = load_or_import(
            &weak,
            data_file.as_deref(),
            index_file.as_deref(),
            game_dir.as_deref(),
            force,
        );
        let result = loaded.and_then(|(dataset, index)| {
            send_progress(
                &weak,
                ImportProgress {
                    stage: if index.is_some() {
                        "Loading EU5 1.3.11 data"
                    } else {
                        "Preparing external data"
                    },
                    current: 1,
                    total: 1,
                },
            );
            state::prepare(dataset, index)
        });
        deliver_result(&weak, result);
    });
}

fn load_or_import(
    weak: &Weak<AppWindow>,
    data_file: Option<&Path>,
    index_file: Option<&Path>,
    game_dir: Option<&Path>,
    force: bool,
) -> Result<(Dataset, Option<StoredFilterIndex>), AppError> {
    if !force && data_file.is_none() {
        return embedded::load().map(|(dataset, index)| (dataset, Some(index)));
    }
    if !force && data_file.is_some_and(Path::is_file) {
        let data_path =
            data_file.ok_or_else(|| AppError::InvalidData("missing data path".to_owned()))?;
        let dataset = storage::load_dataset(data_path)?;
        let index_path = index_file
            .map(Path::to_path_buf)
            .unwrap_or_else(|| inferred_index_path(data_path));
        let index = index_path
            .is_file()
            .then(|| index_storage::load_index(&index_path))
            .transpose()?;
        return Ok((dataset, index));
    }
    let data_path = data_file
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("eu5-locations.bitcode.zst"));
    let index_path = index_file
        .map(Path::to_path_buf)
        .unwrap_or_else(|| inferred_index_path(&data_path));
    let installation = steam::discover(game_dir)?;
    let stored = import_game(&installation, |progress| send_progress(weak, progress))?;
    storage::write_dataset(&data_path, &stored, force)?;
    let dataset = storage::load_dataset(&data_path)?;
    let index = FilterEngine::build_stored_index(&dataset);
    index.validate(&dataset)?;
    index_storage::write_index(&index_path, &index, force)?;
    Ok((dataset, Some(index)))
}

fn inferred_index_path(data_path: &Path) -> PathBuf {
    let mut name = data_path
        .file_stem()
        .map_or_else(|| "eu5-locations".into(), |value| value.to_os_string());
    name.push(".indexes.bitcode.zst");
    data_path.with_file_name(name)
}

fn deliver_result(weak: &Weak<AppWindow>, result: Result<state::PreparedData, AppError>) {
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
