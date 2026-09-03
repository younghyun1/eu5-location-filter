//! Browser event-loop startup and embedded-data loading.

use std::cell::RefCell;
use std::time::Duration;

use slint::{ComponentHandle, SharedString, Timer, Weak};

use super::{AppWindow, state};
use crate::{AppError, embedded};

thread_local! {
    static APPLICATION: RefCell<Option<Weak<AppWindow>>> = const { RefCell::new(None) };
}

pub(super) fn run() -> Result<(), AppError> {
    let app = AppWindow::new().map_err(|error| AppError::Ui(error.to_string()))?;
    app.invoke_apply_theme(true);
    app.set_external_data_controls(false);
    app.set_loading(true);
    app.set_progress_stage("Loading embedded data".into());
    app.set_progress_value(0.0);
    APPLICATION.with(|current| {
        let mut current = current
            .try_borrow_mut()
            .map_err(|error| AppError::Ui(format!("browser application is borrowed: {error}")))?;
        *current = Some(app.as_weak());
        Ok::<(), AppError>(())
    })?;

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

    let result = app.run().map_err(|error| AppError::Ui(error.to_string()));
    APPLICATION.with(|current| {
        if let Ok(mut current) = current.try_borrow_mut() {
            *current = None;
        }
    });
    result
}

pub(super) fn set_theme(theme: &str) -> Result<(), AppError> {
    let dark = match theme {
        "dark" => true,
        "light" => false,
        _ => {
            return Err(AppError::InvalidData(format!(
                "unsupported browser theme: {theme}"
            )));
        }
    };
    let weak = APPLICATION.with(|current| {
        current
            .try_borrow()
            .ok()
            .and_then(|current| current.as_ref().cloned())
    });
    let weak = weak
        .filter(|weak| weak.upgrade().is_some())
        .ok_or_else(|| AppError::Ui("browser application is not running".to_owned()))?;
    slint::invoke_from_event_loop(move || {
        let Some(app) = weak.upgrade() else { return };
        app.invoke_apply_theme(dark);
        app.window().request_redraw();
    })
    .map_err(|error| AppError::Ui(error.to_string()))
}
