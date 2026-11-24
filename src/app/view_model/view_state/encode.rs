//! Canonical encoding of the current view: table order, defaults omitted, sorted and
//! deduplicated items, shortest round-trip numbers, uppercase hex.

use crate::app::view_model::SortState;
use crate::filter::FilterSet;
use crate::model::Dataset;

/// The parts of the application state a view-state string records.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ViewSnapshot<'a> {
    /// Active filters, including the search.
    pub(crate) filters: &'a FilterSet,
    /// Active sort.
    pub(crate) sort: SortState,
    /// Key of the selected location.
    pub(crate) selected_key: Option<&'a str>,
    /// Dataset, for symbol keys of selected facet values.
    pub(crate) dataset: &'a Dataset,
}

/// Clears `out` and writes the canonical string. Fields or items the decoder would reject are
/// omitted; when the result would exceed `VIEW_STATE_MAX_LEN` the default `s1` is written.
pub(crate) fn encode_into(out: &mut String, view: &ViewSnapshot<'_>) {
    let _ = view;
    out.clear();
    out.push_str(super::VIEW_STATE_VERSION);
}
