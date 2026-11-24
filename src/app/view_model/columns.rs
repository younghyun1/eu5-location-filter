//! Column widths and visibility, and the column menu rows.

use super::column_id::ColumnId;
use crate::app::i18n::{Locale, t};
use crate::app::layout::{COLUMN_MAX_WIDTH, COLUMN_MIN_WIDTH, GRID_INLINE_INSET};
use crate::app::widget::ChecklistModel;

/// Width and visibility of one column.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Column {
    /// Column identity.
    pub(crate) id: ColumnId,
    /// Width in logical pixels, within `COLUMN_MIN_WIDTH..=COLUMN_MAX_WIDTH`.
    pub(crate) width: f32,
    /// Whether the grid shows it.
    pub(crate) visible: bool,
}

/// All columns in display order; at least one stays visible.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Columns {
    columns: [Column; ColumnId::COUNT],
}

impl Default for Columns {
    fn default() -> Self {
        Self {
            columns: ColumnId::ALL.map(|id| Column {
                id,
                width: id.default_width(),
                visible: id.default_visible(),
            }),
        }
    }
}

impl Columns {
    /// Every column in display order.
    #[must_use]
    pub(crate) const fn all(&self) -> &[Column; ColumnId::COUNT] {
        &self.columns
    }

    /// One column.
    #[must_use]
    pub(crate) fn get(&self, id: ColumnId) -> Column {
        match self.columns.get(id.index()) {
            Some(column) => *column,
            None => Column {
                id,
                width: id.default_width(),
                visible: false,
            },
        }
    }

    /// Visible columns in display order.
    pub(crate) fn visible(&self) -> impl Iterator<Item = Column> + '_ {
        self.columns.iter().copied().filter(|column| column.visible)
    }

    /// Number of visible columns.
    #[must_use]
    pub(crate) fn visible_count(&self) -> usize {
        self.visible().count()
    }

    /// Shows or hides a column; returns `false` when hiding would leave none visible.
    pub(crate) fn set_visible(&mut self, id: ColumnId, visible: bool) -> bool {
        if !visible && self.get(id).visible && self.visible_count() <= 1 {
            return false;
        }
        if let Some(column) = self.columns.get_mut(id.index()) {
            column.visible = visible;
        }
        true
    }

    /// Sets a width, clamped to `COLUMN_MIN_WIDTH..=COLUMN_MAX_WIDTH`.
    pub(crate) fn set_width(&mut self, id: ColumnId, width: f32) {
        if let Some(column) = self.columns.get_mut(id.index()) {
            column.width = width.clamp(COLUMN_MIN_WIDTH, COLUMN_MAX_WIDTH);
        }
    }

    /// Content width: `GRID_INLINE_INSET` plus every visible width.
    #[must_use]
    pub(crate) fn total_width(&self) -> f32 {
        GRID_INLINE_INSET + self.visible().map(|column| column.width).sum::<f32>()
    }
}

/// Column menu rows: every column in display order, checked when visible.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ColumnRows<'a> {
    /// Column state.
    pub(crate) columns: &'a Columns,
    /// Label locale.
    pub(crate) locale: Locale,
}

impl ChecklistModel for ColumnRows<'_> {
    fn row_count(&self) -> usize {
        ColumnId::COUNT
    }

    fn label(&self, row: usize) -> &str {
        match ColumnId::ALL.get(row) {
            Some(column) => t(self.locale, column.title()),
            None => "",
        }
    }

    fn is_checked(&self, row: usize) -> bool {
        ColumnId::ALL
            .get(row)
            .is_some_and(|column| self.columns.get(*column).visible)
    }

    fn option(&self, row: usize) -> u32 {
        u32::try_from(row).unwrap_or(u32::MAX)
    }
}
