//! Browser event-loop startup and embedded-data loading.

use std::time::Duration;

use slint::{ComponentHandle, SharedString, Timer};

use super::{AppWindow, state};
use crate::{AppError, embedded};

pub(super) fn run() -> Result<(), AppError> {
    let app = AppWindow::new().map_err(|error| AppError::Ui(error.to_string()))?;
    app.set_external_data_controls(false);
    app.set_loading(true);
    app.set_progress_stage("Loading embedded data".into());
    app.set_progress_value(0.0);

    let weak = app.as_weak();
    Timer::single_shot(Duration::ZERO, move || {
        let Some(app) = weak.upgrade() else { return };
        let result = embedded::load().and_then(|(dataset, index)| {
            app.set_progress_stage("Preparing EU5 1.3.11 data".into());
            state::prepare(dataset, Some(index))
        });
        match result {
            Ok(prepared) => state::configure(&app, prepared),
            Err(error) => {
                app.set_loading(false);
                app.set_error_text(SharedString::from(error.to_string()));
            }
        }
    });

    app.run().map_err(|error| AppError::Ui(error.to_string()))
}
