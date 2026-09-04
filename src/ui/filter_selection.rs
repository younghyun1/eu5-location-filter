//! Checkbox-state mapping for categorical filters.

use std::collections::HashSet;
use std::hash::Hash;

use super::state::ActiveState;
use crate::model::{LocationKind, SymbolId};

pub(super) fn is_checked(state: &ActiveState, field: &str, key: &str) -> bool {
    match field {
        "Type" => kind(key).is_some_and(|value| state.filters.kinds.contains(&value)),
        "Topography" => {
            symbol(state, key).is_some_and(|value| state.filters.topographies.contains(&value))
        }
        "Vegetation" => optional_symbol(state, key)
            .is_some_and(|value| state.filters.vegetation.contains(&value)),
        "Climate" => {
            optional_symbol(state, key).is_some_and(|value| state.filters.climates.contains(&value))
        }
        "Continent" => facet_checked(&state.filters.continents, state, key),
        "Subcontinent" => facet_checked(&state.filters.subcontinents, state, key),
        "Region" => facet_checked(&state.filters.regions, state, key),
        "Area" => facet_checked(&state.filters.areas, state, key),
        "Province" => facet_checked(&state.filters.provinces, state, key),
        "Religion" => facet_checked(&state.filters.religions, state, key),
        "Culture" => facet_checked(&state.filters.cultures, state, key),
        "Raw material" => facet_checked(&state.filters.raw_materials, state, key),
        "Modifier" => facet_checked(&state.filters.modifiers, state, key),
        "Coastal" => presence_checked(&state.filters.coastal, key, "yes", "no"),
        "River" => presence_checked(&state.filters.river_presence, key, "present", "missing"),
        "Min river bonus" => numeric_checked(&state.filters.river_level_min, key),
        "Max river bonus" => numeric_checked(&state.filters.river_level_max, key),
        "Harbor suitability" => {
            presence_checked(&state.filters.harbor_presence, key, "present", "missing")
        }
        "Movement assistance" => {
            presence_checked(&state.filters.movement_presence, key, "present", "missing")
        }
        _ => false,
    }
}

pub(super) fn toggle(state: &mut ActiveState, field: &str, key: &str, checked: bool) {
    let resolved = state.resolve(key);
    let optional_resolved = if key == "__missing__" {
        Some(None)
    } else {
        resolved.map(Some)
    };
    match field {
        "Type" => update_set(&mut state.filters.kinds, kind(key), checked),
        "Topography" => {
            let value = symbol(state, key);
            update_set(&mut state.filters.topographies, value, checked);
        }
        "Vegetation" => {
            let value = optional_symbol(state, key);
            update_set(&mut state.filters.vegetation, value, checked);
        }
        "Climate" => {
            let value = optional_symbol(state, key);
            update_set(&mut state.filters.climates, value, checked);
        }
        "Continent" => update_set(&mut state.filters.continents, resolved.map(Some), checked),
        "Subcontinent" => update_set(
            &mut state.filters.subcontinents,
            resolved.map(Some),
            checked,
        ),
        "Region" => update_set(&mut state.filters.regions, resolved.map(Some), checked),
        "Area" => update_set(&mut state.filters.areas, resolved.map(Some), checked),
        "Province" => update_set(&mut state.filters.provinces, resolved.map(Some), checked),
        "Religion" => update_set(&mut state.filters.religions, optional_resolved, checked),
        "Culture" => update_set(&mut state.filters.cultures, optional_resolved, checked),
        "Raw material" => update_set(&mut state.filters.raw_materials, optional_resolved, checked),
        "Modifier" => update_set(&mut state.filters.modifiers, optional_resolved, checked),
        "Coastal" => set_presence(&mut state.filters.coastal, key, "yes", "no", checked),
        "River" => set_presence(
            &mut state.filters.river_presence,
            key,
            "present",
            "missing",
            checked,
        ),
        "Min river bonus" => set_numeric(&mut state.filters.river_level_min, key, checked),
        "Max river bonus" => set_numeric(&mut state.filters.river_level_max, key, checked),
        "Harbor suitability" => set_presence(
            &mut state.filters.harbor_presence,
            key,
            "present",
            "missing",
            checked,
        ),
        "Movement assistance" => set_presence(
            &mut state.filters.movement_presence,
            key,
            "present",
            "missing",
            checked,
        ),
        _ => {}
    }
}

pub(super) fn clear(state: &mut ActiveState, field: &str) {
    match field {
        "Type" => state.filters.kinds.clear(),
        "Topography" => state.filters.topographies.clear(),
        "Vegetation" => state.filters.vegetation.clear(),
        "Climate" => state.filters.climates.clear(),
        "Continent" => state.filters.continents.clear(),
        "Subcontinent" => state.filters.subcontinents.clear(),
        "Region" => state.filters.regions.clear(),
        "Area" => state.filters.areas.clear(),
        "Province" => state.filters.provinces.clear(),
        "Religion" => state.filters.religions.clear(),
        "Culture" => state.filters.cultures.clear(),
        "Raw material" => state.filters.raw_materials.clear(),
        "Modifier" => state.filters.modifiers.clear(),
        "Coastal" => state.filters.coastal.clear(),
        "River" => state.filters.river_presence.clear(),
        "Min river bonus" => state.filters.river_level_min.clear(),
        "Max river bonus" => state.filters.river_level_max.clear(),
        "Harbor suitability" => state.filters.harbor_presence.clear(),
        "Movement assistance" => state.filters.movement_presence.clear(),
        _ => {}
    }
}

fn update_set<T: Eq + Hash>(values: &mut HashSet<T>, value: Option<T>, checked: bool) {
    let Some(value) = value else { return };
    if checked {
        values.insert(value);
    } else {
        values.remove(&value);
    }
}

fn kind(key: &str) -> Option<LocationKind> {
    match key {
        "land" => Some(LocationKind::Land),
        "sea" => Some(LocationKind::Sea),
        "lake" => Some(LocationKind::Lake),
        "impassable" => Some(LocationKind::Impassable),
        "unknown" => Some(LocationKind::Unknown),
        _ => None,
    }
}

fn symbol(state: &ActiveState, key: &str) -> Option<SymbolId> {
    (key != "__missing__").then(|| state.resolve(key)).flatten()
}

fn optional_symbol(state: &ActiveState, key: &str) -> Option<Option<SymbolId>> {
    if key == "__missing__" {
        Some(None)
    } else {
        symbol(state, key).map(Some)
    }
}

fn facet_checked(facet: &HashSet<Option<SymbolId>>, state: &ActiveState, key: &str) -> bool {
    optional_symbol(state, key).is_some_and(|value| facet.contains(&value))
}

fn presence_checked(values: &HashSet<bool>, key: &str, true_key: &str, false_key: &str) -> bool {
    presence_value(key, true_key, false_key).is_some_and(|value| values.contains(&value))
}

fn set_presence(
    values: &mut HashSet<bool>,
    key: &str,
    true_key: &str,
    false_key: &str,
    checked: bool,
) {
    update_set(values, presence_value(key, true_key, false_key), checked);
}

fn presence_value(key: &str, true_key: &str, false_key: &str) -> Option<bool> {
    match key {
        value if value == true_key => Some(true),
        value if value == false_key => Some(false),
        _ => None,
    }
}

fn numeric_checked(values: &HashSet<u8>, key: &str) -> bool {
    key.parse::<u8>()
        .ok()
        .is_some_and(|value| values.contains(&value))
}

fn set_numeric(values: &mut HashSet<u8>, key: &str, checked: bool) {
    update_set(values, key.parse::<u8>().ok(), checked);
}
