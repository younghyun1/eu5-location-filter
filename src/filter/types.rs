//! Public filter selections and ranges.

use std::collections::HashSet;

use crate::AppError;
use crate::model::{LocationKind, MapColor, SymbolId};

/// Selection for a nullable categorical field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OptionalFacet {
    /// Do not constrain the field.
    Any,
    /// Match records without a value.
    Missing,
    /// Match this exact interned value.
    Value(SymbolId),
}

/// Selection for nullable numeric fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OptionalNumeric {
    /// Include missing and present values.
    Any,
    /// Include missing values only.
    Missing,
    /// Include present values subject to any range.
    Present,
}

/// Inclusive floating-point bounds.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FloatRange {
    /// Inclusive minimum.
    pub min: Option<f32>,
    /// Inclusive maximum.
    pub max: Option<f32>,
}

impl FloatRange {
    pub(super) fn matches(self, value: f32) -> bool {
        self.min.is_none_or(|minimum| value >= minimum)
            && self.max.is_none_or(|maximum| value <= maximum)
    }
}

/// Every supported filter. Distinct fields combine with AND.
#[derive(Clone, Debug)]
pub struct FilterSet {
    /// Case-folded name and identifier search.
    pub search: String,
    /// OR-selected location kinds.
    pub kinds: HashSet<LocationKind>,
    /// OR-selected topographies.
    pub topographies: HashSet<SymbolId>,
    /// OR-selected vegetation, including explicit `None`.
    pub vegetation: HashSet<Option<SymbolId>>,
    /// OR-selected climate, including explicit `None`.
    pub climates: HashSet<Option<SymbolId>>,
    /// Exact hierarchy facets.
    pub continent: OptionalFacet,
    /// Exact hierarchy facets.
    pub subcontinent: OptionalFacet,
    /// Exact hierarchy facets.
    pub region: OptionalFacet,
    /// Exact hierarchy facets.
    pub area: OptionalFacet,
    /// Exact hierarchy facets.
    pub province: OptionalFacet,
    /// Exact nullable attribute.
    pub religion: OptionalFacet,
    /// Exact nullable attribute.
    pub culture: OptionalFacet,
    /// Exact nullable attribute.
    pub raw_material: OptionalFacet,
    /// Exact nullable attribute.
    pub modifier: OptionalFacet,
    /// Exact true-color value.
    pub rgb: Option<MapColor>,
    /// Exact coastal state.
    pub coastal: Option<bool>,
    /// Exact river presence.
    pub river_presence: Option<bool>,
    /// Inclusive minimum river level.
    pub river_level_min: Option<u8>,
    /// Inclusive maximum river level.
    pub river_level_max: Option<u8>,
    /// Harbor missing/present selector.
    pub harbor_presence: OptionalNumeric,
    /// Inclusive harbor bounds.
    pub harbor_range: FloatRange,
    /// Exact movement-assistance presence.
    pub movement_presence: Option<bool>,
    /// Inclusive first movement component bounds.
    pub movement_x: FloatRange,
    /// Inclusive second movement component bounds.
    pub movement_y: FloatRange,
    /// Whether impassable locations remain eligible.
    pub show_impassable: bool,
}

impl Default for FilterSet {
    fn default() -> Self {
        Self {
            search: String::new(),
            kinds: HashSet::new(),
            topographies: HashSet::new(),
            vegetation: HashSet::new(),
            climates: HashSet::new(),
            continent: OptionalFacet::Any,
            subcontinent: OptionalFacet::Any,
            region: OptionalFacet::Any,
            area: OptionalFacet::Any,
            province: OptionalFacet::Any,
            religion: OptionalFacet::Any,
            culture: OptionalFacet::Any,
            raw_material: OptionalFacet::Any,
            modifier: OptionalFacet::Any,
            rgb: None,
            coastal: None,
            river_presence: None,
            river_level_min: None,
            river_level_max: None,
            harbor_presence: OptionalNumeric::Any,
            harbor_range: FloatRange::default(),
            movement_presence: None,
            movement_x: FloatRange::default(),
            movement_y: FloatRange::default(),
            show_impassable: true,
        }
    }
}

/// Sortable result-list fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SortField {
    /// English display name.
    Name,
    /// Internal identifier.
    Identifier,
    /// Derived kind.
    Kind,
    /// Topography.
    Topography,
    /// Vegetation, nulls last.
    Vegetation,
    /// Climate, nulls last.
    Climate,
    /// River level, nulls last.
    RiverLevel,
}

/// Parses an optional finite numeric bound for inline validation.
pub fn parse_optional_number(input: &str) -> Result<Option<f32>, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let value = trimmed
        .parse::<f32>()
        .map_err(|error| AppError::InvalidData(format!("invalid number: {error}")))?;
    if !value.is_finite() {
        return Err(AppError::InvalidData("number must be finite".to_owned()));
    }
    Ok(Some(value))
}
