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
#[cfg(test)]
mod result_model_tests;
mod state;
#[cfg(feature = "desktop")]
mod worker;

#[cfg(all(feature = "web", target_family = "wasm"))]
mod web;

#[cfg(feature = "desktop")]
use std::path::PathBuf;

#[cfg(feature = "desktop")]
use slint::ComponentHandle;

#[cfg(feature = "desktop")]
use crate::AppError;

/// Opens the desktop UI and starts loading or importing on a worker thread.
#[cfg(feature = "desktop")]
pub fn run(
    data_file: Option<PathBuf>,
    index_file: Option<PathBuf>,
    game_dir: Option<PathBuf>,
) -> Result<(), AppError> {
    let app = AppWindow::new().map_err(|error| AppError::Ui(error.to_string()))?;
    app.invoke_apply_theme(true);
    worker::install_action_callbacks(
        &app,
        data_file.clone(),
        index_file.clone(),
        game_dir.clone(),
    );
    worker::start_load(app.as_weak(), data_file, index_file, game_dir, false);
    app.run().map_err(|error| AppError::Ui(error.to_string()))
}

/// Starts the Slint application in the browser canvas.
#[cfg(all(feature = "web", target_family = "wasm"))]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start_web() -> Result<(), wasm_bindgen::JsValue> {
    web::run().map_err(|error| wasm_bindgen::JsValue::from_str(&error.to_string()))
}

/// Applies a host-selected color scheme to the active browser application.
#[cfg(all(feature = "web", target_family = "wasm"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn set_web_theme(theme: &str) -> Result<(), wasm_bindgen::JsValue> {
    web::set_theme(theme).map_err(|error| wasm_bindgen::JsValue::from_str(&error.to_string()))
}
