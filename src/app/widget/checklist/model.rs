//! Row source of a checklist.

/// Rows a checklist shows. Implemented by facet rows (visible options of one group, checked
/// against the filters) and by the column menu rows. Row indices are positions in the shown
/// list; `option` maps a row to the stable option index the toggle callback receives.
pub(crate) trait ChecklistModel {
    /// Number of rows shown.
    fn row_count(&self) -> usize;

    /// Label of a row; empty when out of range.
    fn label(&self, row: usize) -> &str;

    /// Whether a row is checked.
    fn is_checked(&self, row: usize) -> bool;

    /// Stable option index of a row.
    fn option(&self, row: usize) -> u32;
}
