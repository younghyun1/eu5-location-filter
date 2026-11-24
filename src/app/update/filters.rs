//! Filter messages, result refresh, and the derived state that follows a result change.

use iced::Task;

use super::fields::{parse_bound, parse_rgb, truncate_to};
use crate::app::message::{FilterMessage, Message};
use crate::app::state::App;
use crate::app::view_model::view_state::MAX_SEARCH_BYTES;
use crate::app::view_model::{self, Bound, FacetId, FacetOptions, RangeField, build_detail};
use crate::filter::{FilterEngine, FilterSet};

/// Applies a filter message. Invalid colour or range text sets the field's inline error and
/// leaves the active filter unchanged; Clear all restores `FilterSet::land_only` and empties
/// every field, the search and each option query.
pub(super) fn filter(app: &mut App, message: FilterMessage) -> Task<Message> {
    let changed = match message {
        FilterMessage::Search(mut text) => {
            // The view-state string carries at most this many bytes, so a longer search could
            // never be shared or restored.
            truncate_to(&mut text, MAX_SEARCH_BYTES);
            if app.filters.search == text {
                false
            } else {
                app.filters.search = text;
                true
            }
        }
        FilterMessage::Toggle {
            facet,
            value,
            selected,
        } => view_model::set_selected(&mut app.filters, facet, value, selected),
        FilterMessage::ClearFacet(facet) => {
            view_model::clear(&mut app.filters, facet);
            true
        }
        FilterMessage::Fold { facet, folded } => {
            if let Some(group) = app.facets.get_mut(facet.index()) {
                group.folded = folded;
            }
            false
        }
        FilterMessage::OptionQuery { facet, query } => {
            set_option_query(app, facet, query);
            false
        }
        FilterMessage::ShowImpassable(value) => {
            std::mem::replace(&mut app.filters.show_impassable, value) != value
        }
        FilterMessage::FoodOnly(value) => {
            std::mem::replace(&mut app.filters.food_producing_only, value) != value
        }
        FilterMessage::Rgb(text) => set_rgb(app, text),
        FilterMessage::Range { field, bound, text } => set_range(app, field, bound, text),
        FilterMessage::ClearAll => {
            clear_all(app);
            true
        }
    };
    if changed {
        refresh_results(app);
    }
    Task::none()
}

/// Stores an option query and recomputes that group's visible options once.
fn set_option_query(app: &mut App, facet: FacetId, mut query: String) {
    truncate_to(&mut query, MAX_SEARCH_BYTES);
    let Some(group) = app.facets.get_mut(facet.index()) else {
        return;
    };
    if group.query == query {
        return;
    }
    group.query = query;
    if let Some(data) = &app.data {
        group.refresh(data.options.get(facet));
    }
}

/// Stores colour text; returns whether the active filter changed.
fn set_rgb(app: &mut App, text: String) -> bool {
    let parsed = parse_rgb(&text);
    app.fields.rgb.text = text;
    match parsed {
        Ok(color) => {
            app.fields.rgb.error = None;
            std::mem::replace(&mut app.filters.rgb, color) != color
        }
        Err(key) => {
            app.fields.rgb.error = Some(key);
            false
        }
    }
}

/// Stores range bound text; returns whether the active filter changed.
fn set_range(app: &mut App, field: RangeField, bound: Bound, text: String) -> bool {
    let parsed = parse_bound(&text);
    let buffer = app.fields.range_mut(field, bound);
    buffer.text = text;
    match parsed {
        Ok(value) => {
            buffer.error = None;
            let range = field.get_mut(&mut app.filters);
            let slot = match bound {
                Bound::Min => &mut range.min,
                Bound::Max => &mut range.max,
            };
            std::mem::replace(slot, value) != value
        }
        Err(key) => {
            buffer.error = Some(key);
            false
        }
    }
}

/// Clear all: default filters, empty rail fields, empty option queries.
fn clear_all(app: &mut App) {
    app.filters = FilterSet::land_only();
    app.fields.clear_rail();
    for group in &mut app.facets {
        group.query.clear();
        if let Some(data) = &app.data {
            group.refresh(data.options.get(group.facet));
        }
    }
}

/// Re-runs `FilterEngine::apply` synchronously, keeps the selection only if it is still in the
/// results, and rebuilds the detail when the selection changed.
pub(super) fn refresh_results(app: &mut App) {
    let Some(data) = app.data.as_mut() else {
        return;
    };
    data.results = data
        .engine
        .apply(&app.filters, app.sort.field, app.sort.ascending);
    let kept = FilterEngine::preserve_selection(app.selection, &data.results);
    if kept != app.selection {
        app.selection = kept;
        rebuild_detail(app);
    }
}

/// Rebuilds the detail pane content for the current selection, locale and language pack.
pub(super) fn rebuild_detail(app: &mut App) {
    app.detail = match (app.selection, app.data.as_ref()) {
        (Some(id), Some(data)) => build_detail(data.engine.labels(), app.locale, id),
        _ => None,
    };
}

/// Rebuilds every option list and its visible indices after a dataset install, a game
/// language change or a locale change (static option labels come from the catalog).
pub(super) fn rebuild_options(app: &mut App) {
    let locale = app.locale;
    let Some(data) = app.data.as_mut() else {
        return;
    };
    data.options = FacetOptions::build(data.engine.labels(), locale);
    for group in &mut app.facets {
        group.refresh(data.options.get(group.facet));
    }
}

/// Everything that follows installing or dropping a language pack: labels, search text and
/// label-dependent sort orders all change, so options, results and detail are rebuilt.
pub(super) fn after_language_change(app: &mut App) {
    rebuild_options(app);
    refresh_results(app);
    rebuild_detail(app);
}
