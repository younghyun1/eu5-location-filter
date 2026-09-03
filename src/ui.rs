//! Slint desktop interface and background loading orchestration.

#![allow(missing_docs)]

mod generated {
    #![allow(
        clippy::all,
        clippy::expect_used,
        clippy::panic,
        clippy::todo,
        clippy::unimplemented,
        clippy::unwrap_used,
        missing_docs
    )]

    slint::include_modules!();
}

use generated::{AppWindow, CheckOption, ColumnSpec, DetailField, LocationRow};

mod column_controls;
mod columns;
mod controls;
mod detail;
mod filter_selection;
mod options;
mod result_model;
mod state;
mod worker;

use std::path::PathBuf;

use slint::ComponentHandle;

use crate::AppError;

/// Opens the desktop UI and starts loading or importing on a worker thread.
pub fn run(
    data_file: Option<PathBuf>,
    index_file: Option<PathBuf>,
    game_dir: Option<PathBuf>,
) -> Result<(), AppError> {
    let app = AppWindow::new().map_err(|error| AppError::Ui(error.to_string()))?;
    worker::install_action_callbacks(
        &app,
        data_file.clone(),
        index_file.clone(),
        game_dir.clone(),
    );
    worker::start_load(app.as_weak(), data_file, index_file, game_dir, false);
    app.run().map_err(|error| AppError::Ui(error.to_string()))
}
