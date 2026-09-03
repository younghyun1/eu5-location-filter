//! Compact numeric attributes stored on locations.

use bitcode::{Decode, Encode};

/// Gameplay river bonus tier, from brook (1) through major river (5).
#[derive(Clone, Copy, Debug, Decode, Encode, Eq, Ord, PartialEq, PartialOrd)]
pub struct RiverLevel(pub u8);

impl RiverLevel {
    /// English gameplay name for this tier.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self.0 {
            1 => "Brook",
            2 => "Stream",
            3 => "River",
            4 => "Large river",
            5 => "Major river",
            _ => "Unknown",
        }
    }
}

/// Population count stored as whole people.
#[derive(Clone, Copy, Debug, Decode, Encode, Eq, Ord, PartialEq, PartialOrd)]
pub struct PopulationAmount(pub u32);

/// Population capacity derived only from immutable vanilla map factors.
#[derive(Clone, Copy, Debug, Decode, Encode, Eq, PartialEq)]
pub struct StaticPopulationCapacity {
    /// Additive capacity supplied by vegetation.
    pub vegetation: PopulationAmount,
    /// Additive latitude contribution from closeness to the equator.
    pub equator: PopulationAmount,
    /// Sum of immutable percentage modifiers in basis points.
    pub modifier_basis_points: i16,
    /// Final capacity after applying the immutable modifiers.
    pub total: PopulationAmount,
}
