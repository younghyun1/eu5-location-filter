//! Per-group checklist state and the checklist rows of a facet group.

use super::facets::FacetId;
use super::options::OptionList;
use super::selection::is_selected;
use crate::app::widget::ChecklistModel;
use crate::filter::FilterSet;

/// Interface state of one facet group.
#[derive(Clone, Debug)]
pub(crate) struct FacetUi {
    /// Group identity.
    pub(crate) facet: FacetId,
    /// Folded groups show only their header and count badge.
    pub(crate) folded: bool,
    /// Option search text.
    pub(crate) query: String,
    /// Indices into the group's `OptionList` that match `query`, in list order; recomputed only
    /// when the query or the option list changes.
    pub(crate) visible: Vec<u32>,
}

impl FacetUi {
    /// Initial state: folded per `starts_folded`, empty query, no options until data loads.
    #[must_use]
    pub(crate) fn new(facet: FacetId) -> Self {
        Self {
            facet,
            folded: facet.starts_folded(),
            query: String::new(),
            visible: Vec::new(),
        }
    }

    /// Recomputes `visible` for the current query against `options`.
    pub(crate) fn refresh(&mut self, options: &OptionList) {
        options.filter_into(&self.query, &mut self.visible);
    }
}

/// Checklist rows of one facet group: the visible options with their checked state.
#[derive(Clone, Copy, Debug)]
pub(crate) struct FacetRows<'a> {
    /// Group identity.
    pub(crate) facet: FacetId,
    /// Full option list of the group.
    pub(crate) options: &'a OptionList,
    /// Visible option indices from `FacetUi::visible`.
    pub(crate) visible: &'a [u32],
    /// Active filters, for checked state.
    pub(crate) filters: &'a FilterSet,
}

impl ChecklistModel for FacetRows<'_> {
    fn row_count(&self) -> usize {
        self.visible.len()
    }

    fn label(&self, row: usize) -> &str {
        match self.visible.get(row) {
            Some(index) => self.options.label(*index),
            None => "",
        }
    }

    fn is_checked(&self, row: usize) -> bool {
        self.visible
            .get(row)
            .and_then(|index| self.options.value(*index))
            .is_some_and(|value| is_selected(self.filters, self.facet, value))
    }

    fn option(&self, row: usize) -> u32 {
        self.visible.get(row).copied().unwrap_or(u32::MAX)
    }
}
