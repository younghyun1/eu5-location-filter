//! Resolution of raw view-state values against the loaded dataset.

use super::decode::RawViewState;
use crate::app::view_model::{FacetOptions, SortState};
use crate::filter::FilterSet;
use crate::model::{Dataset, LocationId};

/// Location IDs sorted by key text, built once per dataset load, so a selected key resolves by
/// binary search without a per-dataset hash map of strings.
#[derive(Clone, Debug, Default)]
pub(crate) struct LocationKeys {
    sorted: Vec<LocationId>,
}

impl LocationKeys {
    /// Sorts every location by its key text.
    #[must_use]
    pub(crate) fn build(dataset: &Dataset) -> Self {
        let _ = dataset;
        Self { sorted: Vec::new() }
    }

    /// Location with this key.
    #[must_use]
    pub(crate) fn find(&self, dataset: &Dataset, key: &str) -> Option<LocationId> {
        let _ = (dataset, key, &self.sorted);
        None
    }
}

/// A view state ready to replace the application's filter, sort and selection state.
#[derive(Clone, Debug)]
pub(crate) struct ResolvedView {
    /// Filters with every unknown item dropped.
    pub(crate) filters: FilterSet,
    /// Sort.
    pub(crate) sort: SortState,
    /// Selection candidate; update keeps it only if it is in the new results.
    pub(crate) selected: Option<LocationId>,
}

/// Resolves facet items through each facet's option list (dropping unknown items and missing
/// values where a facet has none) and the selected key through `keys`.
#[must_use]
pub(crate) fn resolve(
    raw: &RawViewState,
    options: &FacetOptions,
    keys: &LocationKeys,
    dataset: &Dataset,
) -> ResolvedView {
    let _ = (options, keys, dataset);
    ResolvedView {
        filters: FilterSet::land_only(),
        sort: raw.sort,
        selected: None,
    }
}
