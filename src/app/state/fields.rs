//! Text field buffers whose content can be invalid and therefore differs from the filter.

use crate::app::i18n::Text;
use crate::app::view_model::{Bound, RangeField};

/// One text field and its inline error. The active filter changes only when the text parses.
#[derive(Clone, Debug, Default)]
pub(crate) struct FieldBuffer {
    /// Text exactly as typed.
    pub(crate) text: String,
    /// Validation message key shown below the field, if the text is invalid.
    pub(crate) error: Option<Text>,
}

/// Rail and error-card fields. The header search needs no buffer because every string is a
/// valid search; it is `FilterSet::search`.
#[derive(Clone, Debug, Default)]
pub(crate) struct TextFields {
    /// Exact-colour field.
    pub(crate) rgb: FieldBuffer,
    /// Range fields indexed by `RangeField::index` then `Bound::index`.
    pub(crate) ranges: [[FieldBuffer; 2]; RangeField::COUNT],
    /// Desktop installation directory on the load-error card.
    pub(crate) install_path: String,
}

impl TextFields {
    /// One range field buffer.
    #[must_use]
    pub(crate) fn range(&self, field: RangeField, bound: Bound) -> &FieldBuffer {
        let [harbor, movement_x, movement_y] = &self.ranges;
        let [min, max] = match field {
            RangeField::Harbor => harbor,
            RangeField::MovementX => movement_x,
            RangeField::MovementY => movement_y,
        };
        match bound {
            Bound::Min => min,
            Bound::Max => max,
        }
    }

    /// One range field buffer, mutable.
    pub(crate) fn range_mut(&mut self, field: RangeField, bound: Bound) -> &mut FieldBuffer {
        let [harbor, movement_x, movement_y] = &mut self.ranges;
        let [min, max] = match field {
            RangeField::Harbor => harbor,
            RangeField::MovementX => movement_x,
            RangeField::MovementY => movement_y,
        };
        match bound {
            Bound::Min => min,
            Bound::Max => max,
        }
    }

    /// Empties every rail field and error, as Clear all does; keeps the install path.
    pub(crate) fn clear_rail(&mut self) {
        self.rgb = FieldBuffer::default();
        self.ranges = Default::default();
    }
}
