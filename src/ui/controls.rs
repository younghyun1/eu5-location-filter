//! Filter, sort, and selection callback wiring.

use std::cell::RefCell;
use std::rc::Rc;

use slint::{ComponentHandle, ModelRc, SharedString, Weak};

use super::state::ActiveState;
use super::{AppWindow, column_controls, columns, detail, filter_selection, options};
use crate::filter::{FilterSet, parse_optional_number};
use crate::model::MapColor;

pub(super) fn install(app: &AppWindow, state: &Rc<RefCell<ActiveState>>) {
    let weak = app.as_weak();
    let shared = Rc::clone(state);
    app.on_filter_options(move |field, query| {
        let Some(app) = weak.upgrade() else {
            return ModelRc::default();
        };
        let Ok(state) = shared.try_borrow() else {
            return ModelRc::default();
        };
        options::filtered(
            options::source(&app, field.as_str()),
            query.as_str(),
            |key| filter_selection::is_checked(&state, field.as_str(), key),
        )
    });
    let weak = app.as_weak();
    let shared = Rc::clone(state);
    app.on_search_changed(move |value| {
        update(&weak, &shared, |state| state.filters.search = value.into());
    });
    let weak = app.as_weak();
    let shared = Rc::clone(state);
    app.on_toggle_impassable(move |value| {
        update(&weak, &shared, |state| {
            state.filters.show_impassable = value;
        });
    });
    install_values(app, state);
    install_sort_and_selection(app, state);
    column_controls::install(app, state);
    let weak = app.as_weak();
    let shared = Rc::clone(state);
    app.on_option_toggled(move |field, key, checked| {
        update(&weak, &shared, |state| {
            filter_selection::toggle(state, field.as_str(), key.as_str(), checked);
        });
    });
    let weak = app.as_weak();
    let shared = Rc::clone(state);
    app.on_clear_options(move |field| {
        update(&weak, &shared, |state| {
            filter_selection::clear(state, field.as_str());
        });
    });
}

fn install_values(app: &AppWindow, state: &Rc<RefCell<ActiveState>>) {
    install_rgb(app, state);
    let weak = app.as_weak();
    let shared = Rc::clone(state);
    app.on_numeric_changed(move |field, value| {
        validate_numeric(&weak, &shared, field.as_str(), value.as_str());
    });
}

fn install_rgb(app: &AppWindow, state: &Rc<RefCell<ActiveState>>) {
    let weak = app.as_weak();
    let shared = Rc::clone(state);
    app.on_rgb_changed(move |value| {
        let Some(app) = weak.upgrade() else { return };
        let trimmed = value.trim();
        let parsed = if trimmed.is_empty() {
            Ok(None)
        } else {
            MapColor::parse(trimmed).map(Some)
        };
        match parsed {
            Ok(color) => {
                app.set_rgb_error(SharedString::new());
                update_direct(&app, &shared, |state| state.filters.rgb = color);
            }
            Err(error) => app.set_rgb_error(error.to_string().into()),
        }
    });
}

fn install_sort_and_selection(app: &AppWindow, state: &Rc<RefCell<ActiveState>>) {
    let weak = app.as_weak();
    let shared = Rc::clone(state);
    app.on_sort_requested(move |field| {
        update(&weak, &shared, |state| {
            let Some(next) = columns::sort_field(field.as_str()) else {
                return;
            };
            state.ascending = if state.sort == next {
                !state.ascending
            } else {
                true
            };
            state.sort = next;
            state.sort_key = field.to_string();
        });
    });
    let weak = app.as_weak();
    let shared = Rc::clone(state);
    app.on_select_row(move |row| {
        let Some(app) = weak.upgrade() else { return };
        let Ok(row) = usize::try_from(row) else {
            return;
        };
        let Ok(mut state) = shared.try_borrow_mut() else {
            return;
        };
        state.selected = state.model.id_at(row);
        if let Some(id) = state.selected {
            detail::show(&app, &state.dataset, id);
        }
    });
    let weak = app.as_weak();
    let shared = Rc::clone(state);
    app.on_clear_filters(move || {
        let Some(app) = weak.upgrade() else { return };
        app.set_search_text(SharedString::new());
        app.set_show_impassable(true);
        app.set_filter_reset_generation(app.get_filter_reset_generation().wrapping_add(1));
        update_direct(&app, &shared, |state| {
            state.filters = FilterSet::default();
        });
    });
}

fn update(
    weak: &Weak<AppWindow>,
    state: &Rc<RefCell<ActiveState>>,
    change: impl FnOnce(&mut ActiveState),
) {
    let Some(app) = weak.upgrade() else { return };
    update_direct(&app, state, change);
}

fn update_direct(
    app: &AppWindow,
    state: &Rc<RefCell<ActiveState>>,
    change: impl FnOnce(&mut ActiveState),
) {
    let Ok(mut state) = state.try_borrow_mut() else {
        return;
    };
    change(&mut state);
    if let Err(error) = state.refresh(app) {
        app.set_error_text(error.to_string().into());
    }
}

fn validate_numeric(
    weak: &Weak<AppWindow>,
    state: &Rc<RefCell<ActiveState>>,
    field: &str,
    value: &str,
) {
    let Some(app) = weak.upgrade() else { return };
    let parsed = parse_optional_number(value);
    match parsed {
        Ok(value) => {
            app.set_numeric_error(SharedString::new());
            update_direct(&app, state, |state| set_numeric(state, field, value));
        }
        Err(error) => app.set_numeric_error(error.to_string().into()),
    }
}

fn set_numeric(state: &mut ActiveState, field: &str, value: Option<f32>) {
    match field {
        "harbor-min" => state.filters.harbor_range.min = value,
        "harbor-max" => state.filters.harbor_range.max = value,
        "move-x-min" => state.filters.movement_x.min = value,
        "move-x-max" => state.filters.movement_x.max = value,
        "move-y-min" => state.filters.movement_y.min = value,
        "move-y-max" => state.filters.movement_y.max = value,
        _ => {}
    }
}
