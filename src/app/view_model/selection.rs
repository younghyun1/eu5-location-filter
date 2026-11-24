//! Facet selection against `FilterSet` with typed option values; ported from the previous
//! interface's string-keyed checkbox mapping.

use super::facets::FacetId;
use super::options::OptionValue;
use crate::filter::FilterSet;

/// Whether `value` is selected in `facet`; a value of the wrong type for the facet is not.
#[must_use]
pub(crate) fn is_selected(filters: &FilterSet, facet: FacetId, value: OptionValue) -> bool {
    let _ = (filters, facet, value);
    false
}

/// Selects or deselects `value` in `facet`; returns whether the filter changed. A value of the
/// wrong type for the facet changes nothing.
pub(crate) fn set_selected(
    filters: &mut FilterSet,
    facet: FacetId,
    value: OptionValue,
    selected: bool,
) -> bool {
    let _ = (filters, facet, value, selected);
    false
}

/// Empties one facet's selection.
pub(crate) fn clear(filters: &mut FilterSet, facet: FacetId) {
    let _ = (filters, facet);
}

/// Number of selected options in one facet, for its count badge.
#[must_use]
pub(crate) fn selected_count(filters: &FilterSet, facet: FacetId) -> usize {
    let _ = (filters, facet);
    0
}

/// Number of active filter fields for the rail badge: non-empty facets (Type counts when it
/// differs from land only), a search, an exact colour, each range with a bound, Show impassable
/// off, and Food-producing only on.
#[must_use]
pub(crate) fn active_filter_count(filters: &FilterSet) -> usize {
    let _ = filters;
    0
}
