//! Row height helpers shared by the checklist geometry and its callers.

use crate::app::layout::CHECKLIST_ROW_HEIGHT;

/// Height a checklist asks for: `min(rows, max_visible_rows) * row_height`, and one row when
/// the list is empty but has an empty-state text to show.
#[must_use]
pub(in crate::app::widget::checklist) fn list_height(
    rows: usize,
    max_visible_rows: usize,
    row_height: f32,
    has_empty_text: bool,
) -> f32 {
    let shown = rows.min(max_visible_rows);
    let shown = if shown == 0 && has_empty_text {
        1
    } else {
        shown
    };
    rows_to_pixels(shown, sane_row_height(row_height))
}

/// A row height safe to divide by; a non-finite or non-positive value falls back to the wide
/// layout height.
#[must_use]
pub(in crate::app::widget::checklist) fn sane_row_height(row_height: f32) -> f32 {
    if row_height.is_finite() && row_height > 0.0 {
        row_height
    } else {
        CHECKLIST_ROW_HEIGHT
    }
}

/// Pixel length of a number of rows. Row counts stay far below the range where `f32` loses
/// integer precision, so the conversion is exact in practice.
#[must_use]
pub(in crate::app::widget::checklist) fn rows_to_pixels(rows: usize, row_height: f32) -> f32 {
    let rows = rows as f32;
    rows * row_height
}
