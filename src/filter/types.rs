//! Public filter selections and ranges.

use std::collections::HashSet;

use bitcode::{Decode, Encode};

use crate::AppError;
use crate::model::{LocationKind, MapColor, SymbolId};

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
    /// OR-selected continents.
    pub continents: HashSet<Option<SymbolId>>,
    /// OR-selected subcontinents.
    pub subcontinents: HashSet<Option<SymbolId>>,
    /// OR-selected regions.
    pub regions: HashSet<Option<SymbolId>>,
    /// OR-selected areas.
    pub areas: HashSet<Option<SymbolId>>,
    /// OR-selected provinces.
    pub provinces: HashSet<Option<SymbolId>>,
    /// OR-selected religions, including explicit `None`.
    pub religions: HashSet<Option<SymbolId>>,
    /// OR-selected cultures, including explicit `None`.
    pub cultures: HashSet<Option<SymbolId>>,
    /// OR-selected raw materials, including explicit `None`.
    pub raw_materials: HashSet<Option<SymbolId>>,
    /// Restrict raw materials to goods with positive food output.
    pub food_producing_only: bool,
    /// OR-selected modifiers, including explicit `None`.
    pub modifiers: HashSet<Option<SymbolId>>,
    /// Exact true-color value.
    pub rgb: Option<MapColor>,
    /// OR-selected coastal states.
    pub coastal: HashSet<bool>,
    /// OR-selected river-presence states.
    pub river_presence: HashSet<bool>,
    /// OR-selected inclusive minimum gameplay river bonus tiers.
    pub river_level_min: HashSet<u8>,
    /// OR-selected inclusive maximum gameplay river bonus tiers.
    pub river_level_max: HashSet<u8>,
    /// OR-selected harbor missing/present states, represented by presence.
    pub harbor_presence: HashSet<bool>,
    /// Inclusive harbor bounds.
    pub harbor_range: FloatRange,
    /// OR-selected movement-assistance presence states.
    pub movement_presence: HashSet<bool>,
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
            continents: HashSet::new(),
            subcontinents: HashSet::new(),
            regions: HashSet::new(),
            areas: HashSet::new(),
            provinces: HashSet::new(),
            religions: HashSet::new(),
            cultures: HashSet::new(),
            raw_materials: HashSet::new(),
            food_producing_only: false,
            modifiers: HashSet::new(),
            rgb: None,
            coastal: HashSet::new(),
            river_presence: HashSet::new(),
            river_level_min: HashSet::new(),
            river_level_max: HashSet::new(),
            harbor_presence: HashSet::new(),
            harbor_range: FloatRange::default(),
            movement_presence: HashSet::new(),
            movement_x: FloatRange::default(),
            movement_y: FloatRange::default(),
            show_impassable: true,
        }
    }
}

impl FilterSet {
    /// Returns the initial interactive filter state, limited to traversable land.
    #[must_use]
    pub fn land_only() -> Self {
        let mut filters = Self::default();
        filters.kinds.insert(LocationKind::Land);
        filters
    }
}

/// Sortable result-list fields.
#[derive(Clone, Copy, Debug, Decode, Encode, Eq, Hash, PartialEq)]
pub enum SortField {
    /// Exact RGB color.
    Color,
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
    /// Continent.
    Continent,
    /// Subcontinent.
    Subcontinent,
    /// Region.
    Region,
    /// Area.
    Area,
    /// Province.
    Province,
    /// Religion, nulls last.
    Religion,
    /// Culture, nulls last.
    Culture,
    /// Raw material, nulls last.
    RawMaterial,
    /// Modifier, nulls last.
    Modifier,
    /// Coastal state.
    Coastal,
    /// River presence.
    RiverPresence,
    /// Gameplay river bonus tier, nulls last.
    RiverLevel,
    /// Harbor suitability, nulls last.
    HarborSuitability,
    /// Movement-assistance presence.
    MovementPresence,
    /// First movement-assistance component, nulls last.
    MovementX,
    /// Second movement-assistance component, nulls last.
    MovementY,
    /// Immutable population capacity, nulls last.
    StaticPopulationCapacity,
    /// Latitude-based capacity contribution, nulls last.
    EquatorCapacity,
}

impl SortField {
    /// Every supported field, used to build fixed startup indexes.
    pub const ALL: [Self; 25] = [
        Self::Color,
        Self::Name,
        Self::Identifier,
        Self::Kind,
        Self::Topography,
        Self::Vegetation,
        Self::Climate,
        Self::Continent,
        Self::Subcontinent,
        Self::Region,
        Self::Area,
        Self::Province,
        Self::Religion,
        Self::Culture,
        Self::RawMaterial,
        Self::Modifier,
        Self::Coastal,
        Self::RiverPresence,
        Self::RiverLevel,
        Self::HarborSuitability,
        Self::MovementPresence,
        Self::MovementX,
        Self::MovementY,
        Self::StaticPopulationCapacity,
        Self::EquatorCapacity,
    ];
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
