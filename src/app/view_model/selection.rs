//! Facet selection against `FilterSet` with typed option values; ported from the previous
//! interface's string-keyed checkbox mapping.

use std::collections::HashSet;
use std::hash::Hash;

use super::facets::FacetId;
use super::options::OptionValue;
use crate::filter::{FilterSet, FloatRange};
use crate::model::{LocationKind, SymbolId};

/// Whether `value` is selected in `facet`; a value of the wrong type for the facet is not.
#[must_use]
pub(crate) fn is_selected(filters: &FilterSet, facet: FacetId, value: OptionValue) -> bool {
    match (facet, value) {
        (FacetId::Kind, OptionValue::Kind(kind)) => filters.kinds.contains(&kind),
        (FacetId::Topography, OptionValue::Symbol(Some(symbol))) => {
            filters.topographies.contains(&symbol)
        }
        (_, OptionValue::Symbol(symbol)) => {
            optional_set(filters, facet).is_some_and(|set| set.contains(&symbol))
        }
        (_, OptionValue::Flag(flag)) => {
            flag_set(filters, facet).is_some_and(|set| set.contains(&flag))
        }
        (_, OptionValue::Tier(tier)) => {
            tier_set(filters, facet).is_some_and(|set| set.contains(&tier))
        }
        (_, OptionValue::Kind(_)) => false,
    }
}

/// Selects or deselects `value` in `facet`; returns whether the filter changed. A value of the
/// wrong type for the facet changes nothing.
pub(crate) fn set_selected(
    filters: &mut FilterSet,
    facet: FacetId,
    value: OptionValue,
    selected: bool,
) -> bool {
    match (facet, value) {
        (FacetId::Kind, OptionValue::Kind(kind)) => update(&mut filters.kinds, kind, selected),
        (FacetId::Topography, OptionValue::Symbol(Some(symbol))) => {
            update(&mut filters.topographies, symbol, selected)
        }
        (_, OptionValue::Symbol(symbol)) => {
            optional_set_mut(filters, facet).is_some_and(|set| update(set, symbol, selected))
        }
        (_, OptionValue::Flag(flag)) => {
            flag_set_mut(filters, facet).is_some_and(|set| update(set, flag, selected))
        }
        (_, OptionValue::Tier(tier)) => {
            tier_set_mut(filters, facet).is_some_and(|set| update(set, tier, selected))
        }
        (_, OptionValue::Kind(_)) => false,
    }
}

/// Empties one facet's selection.
pub(crate) fn clear(filters: &mut FilterSet, facet: FacetId) {
    match facet {
        FacetId::Kind => filters.kinds.clear(),
        FacetId::Topography => filters.topographies.clear(),
        FacetId::Coastal
        | FacetId::RiverPresence
        | FacetId::HarborPresence
        | FacetId::MovementPresence => {
            if let Some(set) = flag_set_mut(filters, facet) {
                set.clear();
            }
        }
        FacetId::RiverLevelMin | FacetId::RiverLevelMax => {
            if let Some(set) = tier_set_mut(filters, facet) {
                set.clear();
            }
        }
        _ => {
            if let Some(set) = optional_set_mut(filters, facet) {
                set.clear();
            }
        }
    }
}

/// Number of selected options in one facet, for its count badge.
#[must_use]
pub(crate) fn selected_count(filters: &FilterSet, facet: FacetId) -> usize {
    match facet {
        FacetId::Kind => filters.kinds.len(),
        FacetId::Topography => filters.topographies.len(),
        FacetId::Coastal
        | FacetId::RiverPresence
        | FacetId::HarborPresence
        | FacetId::MovementPresence => flag_set(filters, facet).map_or(0, HashSet::len),
        FacetId::RiverLevelMin | FacetId::RiverLevelMax => {
            tier_set(filters, facet).map_or(0, HashSet::len)
        }
        _ => optional_set(filters, facet).map_or(0, HashSet::len),
    }
}

/// Number of active filter fields for the rail badge: non-empty facets (Type counts when it
/// differs from land only), a search, an exact colour, each range with a bound, Show impassable
/// off, and Food-producing only on.
#[must_use]
pub(crate) fn active_filter_count(filters: &FilterSet) -> usize {
    let land_only = filters.kinds.len() == 1 && filters.kinds.contains(&LocationKind::Land);
    let facets = FacetId::ALL
        .into_iter()
        .filter(|facet| match facet {
            FacetId::Kind => !land_only,
            other => selected_count(filters, *other) > 0,
        })
        .count();
    let ranges = [filters.harbor_range, filters.movement_x, filters.movement_y]
        .into_iter()
        .filter(has_bound)
        .count();
    let toggles = [
        !filters.search.trim().is_empty(),
        filters.rgb.is_some(),
        !filters.show_impassable,
        filters.food_producing_only,
    ]
    .into_iter()
    .filter(|active| *active)
    .count();
    facets + ranges + toggles
}

/// Whether a range limits anything.
fn has_bound(range: &FloatRange) -> bool {
    range.min.is_some() || range.max.is_some()
}

/// Inserts or removes `value`; returns whether the set changed.
fn update<T: Eq + Hash>(set: &mut HashSet<T>, value: T, selected: bool) -> bool {
    if selected {
        set.insert(value)
    } else {
        set.remove(&value)
    }
}

/// The facets whose selection holds optional symbols, including the explicit missing value.
fn optional_set(filters: &FilterSet, facet: FacetId) -> Option<&HashSet<Option<SymbolId>>> {
    Some(match facet {
        FacetId::Continent => &filters.continents,
        FacetId::Subcontinent => &filters.subcontinents,
        FacetId::Region => &filters.regions,
        FacetId::Area => &filters.areas,
        FacetId::Province => &filters.provinces,
        FacetId::Vegetation => &filters.vegetation,
        FacetId::Climate => &filters.climates,
        FacetId::Religion => &filters.religions,
        FacetId::Culture => &filters.cultures,
        FacetId::RawMaterial => &filters.raw_materials,
        FacetId::Modifier => &filters.modifiers,
        _ => return None,
    })
}

/// Mutable form of `optional_set`.
fn optional_set_mut(
    filters: &mut FilterSet,
    facet: FacetId,
) -> Option<&mut HashSet<Option<SymbolId>>> {
    Some(match facet {
        FacetId::Continent => &mut filters.continents,
        FacetId::Subcontinent => &mut filters.subcontinents,
        FacetId::Region => &mut filters.regions,
        FacetId::Area => &mut filters.areas,
        FacetId::Province => &mut filters.provinces,
        FacetId::Vegetation => &mut filters.vegetation,
        FacetId::Climate => &mut filters.climates,
        FacetId::Religion => &mut filters.religions,
        FacetId::Culture => &mut filters.cultures,
        FacetId::RawMaterial => &mut filters.raw_materials,
        FacetId::Modifier => &mut filters.modifiers,
        _ => return None,
    })
}

/// The facets whose selection holds presence flags.
fn flag_set(filters: &FilterSet, facet: FacetId) -> Option<&HashSet<bool>> {
    Some(match facet {
        FacetId::Coastal => &filters.coastal,
        FacetId::RiverPresence => &filters.river_presence,
        FacetId::HarborPresence => &filters.harbor_presence,
        FacetId::MovementPresence => &filters.movement_presence,
        _ => return None,
    })
}

/// Mutable form of `flag_set`.
fn flag_set_mut(filters: &mut FilterSet, facet: FacetId) -> Option<&mut HashSet<bool>> {
    Some(match facet {
        FacetId::Coastal => &mut filters.coastal,
        FacetId::RiverPresence => &mut filters.river_presence,
        FacetId::HarborPresence => &mut filters.harbor_presence,
        FacetId::MovementPresence => &mut filters.movement_presence,
        _ => return None,
    })
}

/// The two river tier facets.
fn tier_set(filters: &FilterSet, facet: FacetId) -> Option<&HashSet<u8>> {
    Some(match facet {
        FacetId::RiverLevelMin => &filters.river_level_min,
        FacetId::RiverLevelMax => &filters.river_level_max,
        _ => return None,
    })
}

/// Mutable form of `tier_set`.
fn tier_set_mut(filters: &mut FilterSet, facet: FacetId) -> Option<&mut HashSet<u8>> {
    Some(match facet {
        FacetId::RiverLevelMin => &mut filters.river_level_min,
        FacetId::RiverLevelMax => &mut filters.river_level_max,
        _ => return None,
    })
}
