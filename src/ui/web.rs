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

#[cfg(feature = "web-benchmark")]
pub(super) fn benchmark_filter_work(iterations: u32) -> Result<u32, AppError> {
    use std::sync::Arc;

    use crate::filter::{FilterEngine, FilterSet, SortField};

    if iterations > 10_000 {
        return Err(AppError::InvalidData(
            "browser benchmark iteration count exceeds 10,000".to_owned(),
        ));
    }
    let (dataset, index) = embedded::load()?;
    let engine = FilterEngine::from_stored_index(Arc::new(dataset), index)?;
    let queries = ["a", "river", "kourim", "newyork", "saint"];
    let mut checksum = 0_u32;
    for iteration in 0..iterations {
        let mut filters = FilterSet::land_only();
        let query_index = usize::try_from(iteration)
            .ok()
            .map(|value| value % queries.len())
            .unwrap_or_default();
        filters.search = queries[query_index].to_owned();
        let ids = engine.apply(&filters, SortField::Name, true);
        checksum = checksum.wrapping_add(u32::try_from(ids.len()).unwrap_or(u32::MAX));
    }
    Ok(checksum)
}
