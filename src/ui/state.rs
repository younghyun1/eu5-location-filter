//! UI-owned filtering state and callbacks.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

use slint::{Color, ComponentHandle, ModelRc, SharedString, Weak};

use super::AppWindow;
use super::result_model::{ResultModel, text};
use crate::AppError;
use crate::filter::{
    FilterEngine, FilterSet, OptionalFacet, OptionalNumeric, SortField, parse_optional_number,
};
use crate::model::{Dataset, LocationId, LocationKind, MapColor, SymbolId};

struct ActiveState {
    dataset: Arc<Dataset>,
    engine: FilterEngine,
    model: Rc<ResultModel>,
    filters: FilterSet,
    sort: SortField,
    ascending: bool,
    selected: Option<LocationId>,
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

    fn refresh(&mut self, app: &AppWindow) -> Result<(), AppError> {
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
            clear_detail(app);
        }
        Ok(())
    }

    fn resolve(&self, value: &str) -> Option<SymbolId> {
        self.symbol_lookup
            .get(&value.trim().to_lowercase())
            .copied()
    }
}

pub(super) fn configure(app: &AppWindow, dataset: Dataset) {
    let state = Rc::new(RefCell::new(ActiveState::new(dataset)));
    let model = match state.try_borrow() {
        Ok(state) => state.model.clone(),
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
    install_filter_callbacks(app, &state);
}

fn install_filter_callbacks(app: &AppWindow, state: &Rc<RefCell<ActiveState>>) {
    let weak = app.as_weak();
    let shared = Rc::clone(state);
    app.on_search_changed(move |value| {
        update(&weak, &shared, |state| state.filters.search = value.into())
    });
    let weak = app.as_weak();
    let shared = Rc::clone(state);
    app.on_toggle_impassable(move |value| {
        update(&weak, &shared, |state| {
            state.filters.show_impassable = value
        });
    });
    let weak = app.as_weak();
    let shared = Rc::clone(state);
    app.on_kind_changed(move |value| {
        update(&weak, &shared, |state| {
            state.filters.kinds.clear();
            let kind = match value.as_str() {
                "Land" => Some(LocationKind::Land),
                "Sea" => Some(LocationKind::Sea),
                "Lake" => Some(LocationKind::Lake),
                "Impassable" => Some(LocationKind::Impassable),
                "Unknown" => Some(LocationKind::Unknown),
                _ => None,
            };
            if let Some(kind) = kind {
                state.filters.kinds.insert(kind);
            }
        });
    });
    install_value_callbacks(app, state);
    install_sort_and_selection(app, state);
}

fn install_value_callbacks(app: &AppWindow, state: &Rc<RefCell<ActiveState>>) {
    let weak = app.as_weak();
    let shared = Rc::clone(state);
    app.on_state_changed(move |field, value| {
        update(&weak, &shared, |state| {
            set_presence(state, field.as_str(), value.as_str())
        });
    });
    let weak = app.as_weak();
    let shared = Rc::clone(state);
    app.on_facet_changed(move |field, value| {
        update(&weak, &shared, |state| {
            set_facet(state, field.as_str(), value.as_str())
        });
    });
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
    let weak = app.as_weak();
    let shared = Rc::clone(state);
    app.on_numeric_changed(move |field, value| {
        validate_numeric(&weak, &shared, field.as_str(), value.as_str());
    });
}

fn install_sort_and_selection(app: &AppWindow, state: &Rc<RefCell<ActiveState>>) {
    let weak = app.as_weak();
    let shared = Rc::clone(state);
    app.on_sort_requested(move |field| {
        update(&weak, &shared, |state| {
            let next = match field.as_str() {
                "id" => SortField::Identifier,
                "kind" => SortField::Kind,
                "topography" => SortField::Topography,
                "vegetation" => SortField::Vegetation,
                "climate" => SortField::Climate,
                "river" => SortField::RiverLevel,
                _ => SortField::Name,
            };
            state.ascending = if state.sort == next {
                !state.ascending
            } else {
                true
            };
            state.sort = next;
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
            show_detail(&app, &state.dataset, id);
        }
    });
    let weak = app.as_weak();
    let shared = Rc::clone(state);
    app.on_clear_filters(move || {
        let Some(app) = weak.upgrade() else { return };
        app.set_search_text(SharedString::new());
        app.set_show_impassable(true);
        update_direct(&app, &shared, |state| state.filters = FilterSet::default());
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

fn set_presence(state: &mut ActiveState, field: &str, value: &str) {
    match field {
        "coastal" => state.filters.coastal = yes_no(value),
        "river" => state.filters.river_presence = present_missing(value),
        "movement" => state.filters.movement_presence = present_missing(value),
        "harbor" => {
            state.filters.harbor_presence = match value {
                "Present" => OptionalNumeric::Present,
                "Missing" => OptionalNumeric::Missing,
                _ => OptionalNumeric::Any,
            };
        }
        _ => {}
    }
}

fn set_facet(state: &mut ActiveState, field: &str, value: &str) {
    let trimmed = value.trim();
    let facet = if trimmed.is_empty() {
        OptionalFacet::Any
    } else if trimmed.eq_ignore_ascii_case("missing") {
        OptionalFacet::Missing
    } else if let Some(symbol) = state.resolve(trimmed) {
        OptionalFacet::Value(symbol)
    } else {
        return;
    };
    match field {
        "Continent" => state.filters.continent = facet,
        "Subcontinent" => state.filters.subcontinent = facet,
        "Region" => state.filters.region = facet,
        "Area" => state.filters.area = facet,
        "Province" => state.filters.province = facet,
        "Religion" => state.filters.religion = facet,
        "Culture" => state.filters.culture = facet,
        "Raw material" => state.filters.raw_material = facet,
        "Modifier" => state.filters.modifier = facet,
        "Topography" => set_multi(&mut state.filters.topographies, facet),
        "Vegetation" => set_optional_multi(&mut state.filters.vegetation, facet),
        "Climate" => set_optional_multi(&mut state.filters.climates, facet),
        _ => {}
    }
}

fn set_multi(values: &mut HashSet<SymbolId>, facet: OptionalFacet) {
    values.clear();
    if let OptionalFacet::Value(value) = facet {
        values.insert(value);
    }
}

fn set_optional_multi(values: &mut HashSet<Option<SymbolId>>, facet: OptionalFacet) {
    values.clear();
    match facet {
        OptionalFacet::Missing => {
            values.insert(None);
        }
        OptionalFacet::Value(value) => {
            values.insert(Some(value));
        }
        OptionalFacet::Any => {}
    }
}

fn validate_numeric(
    weak: &Weak<AppWindow>,
    state: &Rc<RefCell<ActiveState>>,
    field: &str,
    value: &str,
) {
    let Some(app) = weak.upgrade() else { return };
    let parsed = if field.starts_with("river-") {
        parse_river_level(value).map(|value| value.map(f32::from))
    } else {
        parse_optional_number(value)
    };
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
        "river-min" => state.filters.river_level_min = value.map(|value| value as u8),
        "river-max" => state.filters.river_level_max = value.map(|value| value as u8),
        "harbor-min" => state.filters.harbor_range.min = value,
        "harbor-max" => state.filters.harbor_range.max = value,
        "move-x-min" => state.filters.movement_x.min = value,
        "move-x-max" => state.filters.movement_x.max = value,
        "move-y-min" => state.filters.movement_y.min = value,
        "move-y-max" => state.filters.movement_y.max = value,
        _ => {}
    }
}

fn parse_river_level(value: &str) -> Result<Option<u8>, AppError> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    value.parse::<u8>().map(Some).map_err(|error| {
        AppError::InvalidData(format!(
            "river level must be an integer from 0 to 255: {error}"
        ))
    })
}

fn yes_no(value: &str) -> Option<bool> {
    match value {
        "Yes" => Some(true),
        "No" => Some(false),
        _ => None,
    }
}

fn present_missing(value: &str) -> Option<bool> {
    match value {
        "Present" => Some(true),
        "Missing" => Some(false),
        _ => None,
    }
}

fn show_detail(app: &AppWindow, dataset: &Dataset, id: LocationId) {
    let Some(record) = dataset.location(id) else {
        return;
    };
    let [red, green, blue] = record.color.components();
    app.set_detail_color(Color::from_rgb_u8(red, green, blue));
    app.set_detail_name(text(dataset, Some(record.name)).into());
    app.set_detail_key(text(dataset, Some(record.key)).into());
    app.set_detail_hex(record.color.hex().into());
    app.set_detail_rgb(format!("rgb({red}, {green}, {blue})").into());
    let breadcrumb = [
        record.hierarchy.continent,
        record.hierarchy.subcontinent,
        record.hierarchy.region,
        record.hierarchy.area,
        record.hierarchy.province,
    ]
    .map(|symbol| dataset.label(symbol).unwrap_or("—"))
    .join("  ›  ");
    app.set_detail_breadcrumb(breadcrumb.into());
    app.set_detail_fields(detail_fields(dataset, record).into());
}

fn detail_fields(dataset: &Dataset, record: &crate::model::LocationRecord) -> String {
    let connected = record
        .connected_sea
        .and_then(|id| dataset.location(id))
        .map_or("—", |sea| text(dataset, Some(sea.key)));
    let river = record.river.map_or_else(
        || "Missing".to_owned(),
        |value| {
            format!(
                "Level {}  •  width {:.2}  •  source {}  •  confluence {}",
                value.level.0, value.rendered_width, value.has_source, value.has_confluence
            )
        },
    );
    let harbor = record
        .harbor_suitability
        .map_or_else(|| "—".to_owned(), |value| format!("{value:.2}"));
    let movement = record.movement_assistance.map_or_else(
        || "—".to_owned(),
        |value| format!("{:.2}, {:.2}", value[0], value[1]),
    );
    format!(
        "Kind\n{}\n\nTopography\n{}\n\nVegetation\n{}\n\nClimate\n{}\n\nReligion\n{}\n\nCulture\n{}\n\nRaw material\n{}\n\nModifier\n{}\n\nCoastal\n{}\n\nConnected sea\n{}\n\nHarbor suitability\n{}\n\nMovement assistance\n{}\n\nRiver\n{}",
        record.kind.label(),
        text(dataset, Some(record.topography)),
        text(dataset, record.vegetation),
        text(dataset, record.climate),
        text(dataset, record.religion),
        text(dataset, record.culture),
        text(dataset, record.raw_material),
        text(dataset, record.modifier),
        record.coastal,
        connected,
        harbor,
        movement,
        river
    )
}

fn clear_detail(app: &AppWindow) {
    app.set_detail_color(Color::from_rgb_u8(32, 40, 50));
    app.set_detail_name("Select a location".into());
    app.set_detail_key(SharedString::new());
    app.set_detail_breadcrumb(SharedString::new());
    app.set_detail_fields(SharedString::new());
}
