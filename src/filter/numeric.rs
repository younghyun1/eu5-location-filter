//! Sorted static numeric values with inclusive binary-searched range bounds.

use bitcode::{Decode, Encode};

use super::{FloatRange, bitmap::Bitmap};
use crate::{
    AppError,
    model::{Dataset, LocationId, LocationRecord},
};

#[derive(Clone, Copy, Debug, Decode, Encode, Eq, PartialEq)]
pub(super) enum NumericField {
    River,
    Harbor,
    MovementX,
    MovementY,
}

impl NumericField {
    pub(super) const ALL: [Self; 4] = [Self::River, Self::Harbor, Self::MovementX, Self::MovementY];

    fn value(self, record: &LocationRecord) -> Option<f32> {
        match self {
            Self::River => record.river.map(|river| f32::from(river.level.0)),
            Self::Harbor => record.harbor_suitability,
            Self::MovementX => record.movement_assistance.map(|vector| vector[0]),
            Self::MovementY => record.movement_assistance.map(|vector| vector[1]),
        }
    }
}

#[derive(Clone, Debug, Decode, Encode, PartialEq)]
pub(super) struct NumericEntry {
    pub(super) value: f32,
    pub(super) id: LocationId,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq)]
pub(super) struct NumericIndex {
    pub(super) field: NumericField,
    pub(super) entries: Vec<NumericEntry>,
}

impl NumericIndex {
    pub(super) fn build(dataset: &Dataset, field: NumericField) -> Self {
        let mut entries: Vec<_> = dataset
            .stored
            .locations
            .iter()
            .filter_map(|record| {
                field.value(record).map(|value| NumericEntry {
                    value,
                    id: record.id,
                })
            })
            .collect();
        entries.sort_unstable_by(|left, right| {
            left.value
                .total_cmp(&right.value)
                .then(left.id.cmp(&right.id))
        });
        Self { field, entries }
    }

    pub(super) fn union_range(&self, range: FloatRange, mask: &mut Bitmap) {
        // NaN and reversed bounds must match nothing, just as the record predicate does.
        if range.min.is_some_and(f32::is_nan) || range.max.is_some_and(f32::is_nan) {
            return;
        }
        let start = range.min.map_or(0, |min| {
            self.entries.partition_point(|entry| entry.value < min)
        });
        let end = range.max.map_or(self.entries.len(), |max| {
            self.entries.partition_point(|entry| entry.value <= max)
        });
        if let Some(entries) = self.entries.get(start..end) {
            for entry in entries {
                mask.insert(entry.id);
            }
        }
    }

    pub(super) fn validate(&self, dataset: &Dataset) -> Result<(), AppError> {
        let mut seen = Bitmap::empty(dataset.stored.locations.len());
        for entry in &self.entries {
            if !entry.value.is_finite()
                || seen.contains(entry.id)
                || dataset
                    .location(entry.id)
                    .and_then(|record| self.field.value(record))
                    != Some(entry.value)
            {
                return Err(AppError::InvalidData(
                    "numeric index has invalid values or duplicate IDs".to_owned(),
                ));
            }
            seen.insert(entry.id);
        }
        if self.entries.windows(2).any(|pair| {
            !matches!(pair, [left, right] if
            left.value.total_cmp(&right.value).then(left.id.cmp(&right.id)).is_lt())
        }) || self.entries.len()
            != dataset
                .stored
                .locations
                .iter()
                .filter(|record| self.field.value(record).is_some())
                .count()
        {
            return Err(AppError::InvalidData(
                "numeric index is unordered or incomplete".to_owned(),
            ));
        }
        Ok(())
    }
}
