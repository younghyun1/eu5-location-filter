//! Numeric range rows of the filter rail.

use crate::app::i18n::Text;
use crate::filter::{FilterSet, FloatRange};

/// A numeric range row.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum RangeField {
    /// Harbor suitability, `FilterSet::harbor_range`.
    Harbor,
    /// First movement component, `FilterSet::movement_x`.
    MovementX,
    /// Second movement component, `FilterSet::movement_y`.
    MovementY,
}

impl RangeField {
    /// Number of range rows.
    pub(crate) const COUNT: usize = 3;

    /// Every row in rail order.
    pub(crate) const ALL: [Self; Self::COUNT] = [Self::Harbor, Self::MovementX, Self::MovementY];

    /// Position in `ALL`.
    #[must_use]
    pub(crate) const fn index(self) -> usize {
        self as usize
    }

    /// Stable key used in widget identifiers.
    #[must_use]
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::Harbor => "harbor",
            Self::MovementX => "movement_x",
            Self::MovementY => "movement_y",
        }
    }

    /// Row label.
    #[must_use]
    pub(crate) const fn title(self) -> Text {
        match self {
            Self::Harbor => Text::RangeHarbor,
            Self::MovementX => Text::RangeMovementX,
            Self::MovementY => Text::RangeMovementY,
        }
    }

    /// Active range of this row.
    #[must_use]
    pub(crate) const fn get(self, filters: &FilterSet) -> FloatRange {
        match self {
            Self::Harbor => filters.harbor_range,
            Self::MovementX => filters.movement_x,
            Self::MovementY => filters.movement_y,
        }
    }

    /// Mutable active range of this row.
    pub(crate) fn get_mut(self, filters: &mut FilterSet) -> &mut FloatRange {
        match self {
            Self::Harbor => &mut filters.harbor_range,
            Self::MovementX => &mut filters.movement_x,
            Self::MovementY => &mut filters.movement_y,
        }
    }
}

/// Minimum or maximum field of a range row.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Bound {
    /// Inclusive minimum.
    Min,
    /// Inclusive maximum.
    Max,
}

impl Bound {
    /// Position in a `[min, max]` pair.
    #[must_use]
    pub(crate) const fn index(self) -> usize {
        self as usize
    }

    /// Stable key used in widget identifiers.
    #[must_use]
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::Min => "min",
            Self::Max => "max",
        }
    }

    /// Field placeholder.
    #[must_use]
    pub(crate) const fn placeholder(self) -> Text {
        match self {
            Self::Min => Text::RangeMinPlaceholder,
            Self::Max => Text::RangeMaxPlaceholder,
        }
    }
}
