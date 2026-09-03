//! Checkbox-state mapping for categorical filters.

use std::collections::HashSet;
use std::hash::Hash;

use super::state::ActiveState;
use crate::filter::{OptionalFacet, OptionalNumeric};
use crate::model::{LocationKind, SymbolId};

pub(super) fn is_checked(state: &ActiveState, field: &str, key: &str) -> bool {
    match field {
        "Kind" => kind(key).is_some_and(|value| state.filters.kinds.contains(&value)),
        "Topography" => {
            symbol(state, key).is_some_and(|value| state.filters.topographies.contains(&value))
        }
        "Vegetation" => optional_symbol(state, key)
            .is_some_and(|value| state.filters.vegetation.contains(&value)),
        "Climate" => {
            optional_symbol(state, key).is_some_and(|value| state.filters.climates.contains(&value))
        }
        "Continent" => facet_checked(state.filters.continent, state, key),
        "Subcontinent" => facet_checked(state.filters.subcontinent, state, key),
        "Region" => facet_checked(state.filters.region, state, key),
        "Area" => facet_checked(state.filters.area, state, key),
        "Province" => facet_checked(state.filters.province, state, key),
        "Religion" => facet_checked(state.filters.religion, state, key),
        "Culture" => facet_checked(state.filters.culture, state, key),
        "Raw material" => facet_checked(state.filters.raw_material, state, key),
        "Modifier" => facet_checked(state.filters.modifier, state, key),
        "Coastal" => presence_checked(state.filters.coastal, key, "yes", "no"),
        "River" => presence_checked(state.filters.river_presence, key, "present", "missing"),
        "Min river bonus" => numeric_checked(state.filters.river_level_min, key),
        "Max river bonus" => numeric_checked(state.filters.river_level_max, key),
        "Harbor suitability" => numeric_presence_checked(state.filters.harbor_presence, key),
        "Movement assistance" => {
            presence_checked(state.filters.movement_presence, key, "present", "missing")
        }
        _ => false,
    }
}

pub(super) fn toggle(state: &mut ActiveState, field: &str, key: &str, checked: bool) {
    let resolved = state.resolve(key);
    match field {
        "Kind" => update_set(&mut state.filters.kinds, kind(key), checked),
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
        "Continent" => set_facet(&mut state.filters.continent, resolved, key, checked),
        "Subcontinent" => set_facet(&mut state.filters.subcontinent, resolved, key, checked),
        "Region" => set_facet(&mut state.filters.region, resolved, key, checked),
        "Area" => set_facet(&mut state.filters.area, resolved, key, checked),
        "Province" => set_facet(&mut state.filters.province, resolved, key, checked),
        "Religion" => set_facet(&mut state.filters.religion, resolved, key, checked),
        "Culture" => set_facet(&mut state.filters.culture, resolved, key, checked),
        "Raw material" => set_facet(&mut state.filters.raw_material, resolved, key, checked),
        "Modifier" => set_facet(&mut state.filters.modifier, resolved, key, checked),
        "Coastal" => set_presence(&mut state.filters.coastal, key, "yes", checked),
        "River" => set_presence(&mut state.filters.river_presence, key, "present", checked),
        "Min river bonus" => set_numeric(&mut state.filters.river_level_min, key, checked),
        "Max river bonus" => set_numeric(&mut state.filters.river_level_max, key, checked),
        "Harbor suitability" => {
            set_numeric_presence(&mut state.filters.harbor_presence, key, checked)
        }
        "Movement assistance" => set_presence(
            &mut state.filters.movement_presence,
            key,
            "present",
            checked,
        ),
        _ => {}
    }
}

pub(super) fn clear(state: &mut ActiveState, field: &str) {
    match field {
        "Kind" => state.filters.kinds.clear(),
        "Topography" => state.filters.topographies.clear(),
        "Vegetation" => state.filters.vegetation.clear(),
        "Climate" => state.filters.climates.clear(),
        "Continent" => state.filters.continent = OptionalFacet::Any,
        "Subcontinent" => state.filters.subcontinent = OptionalFacet::Any,
        "Region" => state.filters.region = OptionalFacet::Any,
        "Area" => state.filters.area = OptionalFacet::Any,
        "Province" => state.filters.province = OptionalFacet::Any,
        "Religion" => state.filters.religion = OptionalFacet::Any,
        "Culture" => state.filters.culture = OptionalFacet::Any,
        "Raw material" => state.filters.raw_material = OptionalFacet::Any,
        "Modifier" => state.filters.modifier = OptionalFacet::Any,
        "Coastal" => state.filters.coastal = None,
        "River" => state.filters.river_presence = None,
        "Min river bonus" => state.filters.river_level_min = None,
        "Max river bonus" => state.filters.river_level_max = None,
        "Harbor suitability" => state.filters.harbor_presence = OptionalNumeric::Any,
        "Movement assistance" => state.filters.movement_presence = None,
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

fn facet_checked(facet: OptionalFacet, state: &ActiveState, key: &str) -> bool {
    match facet {
        OptionalFacet::Any => false,
        OptionalFacet::Missing => key == "__missing__",
        OptionalFacet::Value(value) => symbol(state, key) == Some(value),
    }
}

fn set_facet(facet: &mut OptionalFacet, value: Option<SymbolId>, key: &str, checked: bool) {
    let next = if key == "__missing__" {
        OptionalFacet::Missing
    } else if let Some(value) = value {
        OptionalFacet::Value(value)
    } else {
        return;
    };
    if checked {
        *facet = next;
    } else if *facet == next {
        *facet = OptionalFacet::Any;
    }
}

fn presence_checked(value: Option<bool>, key: &str, true_key: &str, false_key: &str) -> bool {
    value == Some(key == true_key) && (key == true_key || key == false_key)
}

fn set_presence(value: &mut Option<bool>, key: &str, true_key: &str, checked: bool) {
    let Some(next) =
        (key == true_key || key == "no" || key == "missing").then_some(key == true_key)
    else {
        return;
    };
    if checked {
        *value = Some(next);
    } else if *value == Some(next) {
        *value = None;
    }
}

fn numeric_checked(value: Option<u8>, key: &str) -> bool {
    key.parse::<u8>().ok() == value
}

fn set_numeric(value: &mut Option<u8>, key: &str, checked: bool) {
    let Ok(next) = key.parse::<u8>() else { return };
    if checked {
        *value = Some(next);
    } else if *value == Some(next) {
        *value = None;
    }
}

fn numeric_presence_checked(value: OptionalNumeric, key: &str) -> bool {
    matches!(
        (value, key),
        (OptionalNumeric::Present, "present") | (OptionalNumeric::Missing, "missing")
    )
}

fn set_numeric_presence(value: &mut OptionalNumeric, key: &str, checked: bool) {
    let next = match key {
        "present" => OptionalNumeric::Present,
        "missing" => OptionalNumeric::Missing,
        _ => return,
    };
    if checked {
        *value = next;
    } else if *value == next {
        *value = OptionalNumeric::Any;
    }
}
