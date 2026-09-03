//! UI-owned dataset, filtering state, and result model.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use slint::{ModelRc, SharedString};

use super::result_model::ResultModel;
use super::{AppWindow, controls, detail};
use crate::AppError;
use crate::filter::{FilterEngine, FilterSet, SortField};
use crate::model::{Dataset, LocationId, SymbolId};

pub(super) struct ActiveState {
    pub(super) dataset: Arc<Dataset>,
    pub(super) engine: FilterEngine,
    pub(super) model: Rc<ResultModel>,
    pub(super) filters: FilterSet,
    pub(super) sort: SortField,
    pub(super) ascending: bool,
    pub(super) selected: Option<LocationId>,
    symbol_lookup: HashMap<String, SymbolId>,
}

impl ActiveState {
    fn new(dataset: Dataset) -> Self {
        let dataset = Arc::new(dataset);
        let engine = FilterEngine::new(Arc::clone(&dataset));
        let filters = FilterSet::default();
        let ids = engine.apply(&filters, SortField::Name, true);
        let model = ResultModel::new(&dataset, ids);
        let mut symbol_lookup =
            HashMap::with_capacity(dataset.stored.dictionary.len().saturating_mul(2));
        for (index, value) in dataset.stored.dictionary.iter().enumerate() {
            if let Ok(index) = u32::try_from(index) {
                symbol_lookup
                    .entry(value.to_lowercase())
                    .or_insert(SymbolId(index));
            }
        }
        for localized in &dataset.stored.localizations {
            if let Some(value) = dataset.symbol(localized.value) {
                symbol_lookup
                    .entry(value.to_lowercase())
                    .or_insert(localized.key);
            }
        }
        Self {
            dataset,
            engine,
            model,
            filters,
            sort: SortField::Name,
            ascending: true,
            selected: None,
            symbol_lookup,
        }
    }

    pub(super) fn refresh(&mut self, app: &AppWindow) -> Result<(), AppError> {
        let ids = self.engine.apply(&self.filters, self.sort, self.ascending);
        self.selected = FilterEngine::preserve_selection(self.selected, &ids);
        let count = ids.len();
        self.model.reset(ids)?;
        app.set_result_count(format!("{count} locations").into());
        let selected_index = self
            .selected
            .and_then(|id| self.model.position(id))
            .and_then(|value| i32::try_from(value).ok())
            .unwrap_or(-1);
        app.set_selected_index(selected_index);
        if self.selected.is_none() {
            detail::clear(app);
        }
        Ok(())
    }

    pub(super) fn resolve(&self, value: &str) -> Option<SymbolId> {
        self.symbol_lookup
            .get(&value.trim().to_lowercase())
            .copied()
    }
}

pub(super) fn configure(app: &AppWindow, dataset: Dataset) {
    let state = Rc::new(RefCell::new(ActiveState::new(dataset)));
    let model = match state.try_borrow() {
        Ok(state_ref) => state_ref.model.clone(),
        Err(_) => return,
    };
    app.set_rows(ModelRc::from(model));
    if let Ok(state_ref) = state.try_borrow() {
        app.set_build_text(format!("Build {}", state_ref.dataset.stored.build_id).into());
    }
    app.set_error_text(SharedString::new());
    app.set_loading(false);
    if let Ok(mut state_ref) = state.try_borrow_mut() {
        let _ = state_ref.refresh(app);
    }
    controls::install(app, &state);
}
