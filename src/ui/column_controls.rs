//! Result-column visibility and width callbacks.

use std::cell::RefCell;
use std::rc::Rc;

use slint::ComponentHandle;

use super::state::ActiveState;
use super::{AppWindow, columns};

pub(super) fn install(app: &AppWindow, state: &Rc<RefCell<ActiveState>>) {
    let weak = app.as_weak();
    let shared = Rc::clone(state);
    app.on_toggle_column(move |key, visible| {
        let Some(app) = weak.upgrade() else { return };
        let Ok(state) = shared.try_borrow() else {
            return;
        };
        columns::set_visible(&state.columns, key.as_str(), visible);
        app.set_table_width(columns::total_width(&state.columns));
    });
    let weak = app.as_weak();
    let shared = Rc::clone(state);
    app.on_resize_column(move |key, width| {
        let Some(app) = weak.upgrade() else { return };
        let Ok(state) = shared.try_borrow() else {
            return;
        };
        columns::set_width(&state.columns, key.as_str(), width);
        app.set_table_width(columns::total_width(&state.columns));
    });
}
