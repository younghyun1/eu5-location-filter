//! Compact typed records stored in the data blob.

use std::collections::HashMap;

use bitcode::{Decode, Encode};

use crate::AppError;

mod attributes;
mod interner;
mod raw_materials;

pub use attributes::{PopulationAmount, RiverLevel, StaticPopulationCapacity};
pub use interner::StringInterner;
#[cfg(test)]
pub(crate) use raw_materials::raw_material_icon;
pub(crate) use raw_materials::{is_food_producing, is_gold_or_silver, raw_material_display};

/// Current on-disk schema version.
pub const FORMAT_VERSION: u16 = 3;
/// Steam application identifier for Europa Universalis V.
pub const EU5_APP_ID: u32 = 3_450_310;
/// Europa Universalis V version represented by the committed bundles.
pub const EU5_GAME_VERSION: &str = "1.3.11";
/// Upper bound that prevents hostile blobs from growing startup memory without limit.
pub const MAX_SYMBOLS: usize = 200_000;
/// Upper bound for dictionary text in bytes.
pub const MAX_DICTIONARY_BYTES: usize = 32 * 1024 * 1024;
/// Upper bound for imported locations.
pub const MAX_LOCATIONS: usize = 100_000;
/// Highest gameplay river tier defined by EU5 1.3.11.
pub const MAX_RIVER_LEVEL: u8 = 5;

/// Index into the dataset-wide string dictionary.
#[derive(Clone, Copy, Debug, Decode, Encode, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SymbolId(pub u32);

/// Stable location index within one dataset.
#[derive(Clone, Copy, Debug, Decode, Encode, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LocationId(pub u32);

/// True-color RGB value packed as `0xRRGGBB`.
#[derive(Clone, Copy, Debug, Decode, Encode, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MapColor(pub u32);

impl MapColor {
    /// Parses one to six hexadecimal digits, accepting omitted leading zeroes.
    pub fn parse(value: &str) -> Result<Self, AppError> {
        let trimmed = value.trim().trim_start_matches('#');
        if trimmed.is_empty()
            || trimmed.len() > 6
            || !trimmed.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(AppError::InvalidData(format!(
                "RGB value must contain one to six hexadecimal digits: {value}"
            )));
        }
        u32::from_str_radix(trimmed, 16)
            .map(Self)
            .map_err(|error| AppError::InvalidData(format!("invalid RGB value {value}: {error}")))
    }

    /// Returns the red, green, and blue components.
    #[must_use]
    pub const fn components(self) -> [u8; 3] {
        [
            ((self.0 >> 16) & 0xff) as u8,
            ((self.0 >> 8) & 0xff) as u8,
            (self.0 & 0xff) as u8,
        ]
    }

    /// Formats the color as six uppercase hexadecimal digits.
    #[must_use]
    pub fn hex(self) -> String {
        format!("#{:06X}", self.0)
    }
}

/// Broad map-location classification derived from topography.
#[derive(Clone, Copy, Debug, Decode, Encode, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LocationKind {
    /// Traversable land.
    Land,
    /// Ocean, sea, or narrows.
    Sea,
    /// Inland lake.
    Lake,
    /// Any wasteland or otherwise non-traversable land topography.
    Impassable,
    /// A future topography not known to this importer.
    Unknown,
}

impl LocationKind {
    /// Derives a kind while retaining unknown topography in the record itself.
    #[must_use]
    pub fn from_topography(value: &str) -> Self {
        if value.ends_with("_wasteland") || value == "salt_pans" {
            return Self::Impassable;
        }
        match value {
            "lakes" | "high_lakes" => Self::Lake,
            "coastal_ocean" | "deep_ocean" | "inland_sea" | "narrows" | "ocean" => Self::Sea,
            "atoll" | "flatland" | "hills" | "mountains" | "plateau" | "wetlands" => Self::Land,
            _ => Self::Unknown,
        }
    }

    /// User-facing label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Land => "Land",
            Self::Sea => "Sea",
            Self::Lake => "Lake",
            Self::Impassable => "Impassable",
            Self::Unknown => "Unknown",
        }
    }
}

/// Continent-to-province membership for one location.
#[derive(Clone, Copy, Debug, Decode, Encode, Eq, PartialEq)]
pub struct Hierarchy {
    /// Continent symbol.
    pub continent: SymbolId,
    /// Subcontinent symbol.
    pub subcontinent: SymbolId,
    /// Region symbol.
    pub region: SymbolId,
    /// Area symbol.
    pub area: SymbolId,
    /// Province symbol.
    pub province: SymbolId,
}

/// River properties derived by streaming paired map images.
#[derive(Clone, Copy, Debug, Decode, Encode, PartialEq)]
pub struct RiverData {
    /// Highest gameplay bonus tier touching the location.
    pub level: RiverLevel,
    /// Whether palette index 1 touches the location.
    pub has_source: bool,
    /// Whether palette index 2 touches the location.
    pub has_confluence: bool,
}

/// One imported location represented only by typed values and dictionary IDs.
#[derive(Clone, Debug, Decode, Encode, PartialEq)]
pub struct LocationRecord {
    /// Stable index.
    pub id: LocationId,
    /// Internal identifier.
    pub key: SymbolId,
    /// English name or generated fallback.
    pub name: SymbolId,
    /// Derived map kind.
    pub kind: LocationKind,
    /// Exact map color.
    pub color: MapColor,
    /// Topography identifier.
    pub topography: SymbolId,
    /// Optional vegetation identifier.
    pub vegetation: Option<SymbolId>,
    /// Optional climate identifier.
    pub climate: Option<SymbolId>,
    /// Optional religion identifier.
    pub religion: Option<SymbolId>,
    /// Optional culture identifier.
    pub culture: Option<SymbolId>,
    /// Optional raw-material identifier.
    pub raw_material: Option<SymbolId>,
    /// Optional static modifier identifier.
    pub modifier: Option<SymbolId>,
    /// Natural harbor suitability.
    pub harbor_suitability: Option<f32>,
    /// Optional movement-assistance vector.
    pub movement_assistance: Option<[f32; 2]>,
    /// Geographic membership.
    pub hierarchy: Hierarchy,
    /// Whether the location appears as land in `ports.csv`.
    pub coastal: bool,
    /// Connected sea location, when coastal.
    pub connected_sea: Option<LocationId>,
    /// Derived river data.
    pub river: Option<RiverData>,
    /// Capacity calculated without mutable campaign state.
    pub static_population_capacity: Option<StaticPopulationCapacity>,
}

/// Parameters used to map river palette levels to rendered widths.
#[derive(Clone, Copy, Debug, Decode, Encode, PartialEq)]
pub struct RiverWidthMetadata {
    /// Count of width palette values.
    pub level_count: u8,
    /// Width of the lowest river level.
    pub width_min: f32,
    /// Width of the highest river level.
    pub width_max: f32,
}

/// A diagnostic that does not make an import unusable.
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct ImportDiagnostic {
    /// Stable diagnostic code.
    pub code: String,
    /// Count of affected source records.
    pub count: u32,
}

/// Localized label for one internal dictionary symbol.
#[derive(Clone, Copy, Debug, Decode, Encode, Eq, PartialEq)]
pub struct LocalizedValue {
    /// Internal identifier.
    pub key: SymbolId,
    /// English label.
    pub value: SymbolId,
}

/// Serializable dataset payload.
#[derive(Clone, Debug, Decode, Encode, PartialEq)]
pub struct StoredDataset {
    /// Storage schema version.
    pub format_version: u16,
    /// Steam application identifier.
    pub app_id: u32,
    /// Steam build identifier.
    pub build_id: u64,
    /// River rendering parameters.
    pub river_widths: RiverWidthMetadata,
    /// Shared string dictionary.
    pub dictionary: Vec<String>,
    /// English labels retained only for referenced identifiers.
    pub localizations: Vec<LocalizedValue>,
    /// Imported records.
    pub locations: Vec<LocationRecord>,
    /// Non-fatal import findings.
    pub diagnostics: Vec<ImportDiagnostic>,
}

/// Validated in-memory data and bounded startup indexes.
#[derive(Clone, Debug)]
pub struct Dataset {
    /// Stored payload.
    pub stored: StoredDataset,
    /// Internal identifier to location index.
    pub by_key: HashMap<SymbolId, LocationId>,
    /// RGB color to location index.
    pub by_color: HashMap<MapColor, LocationId>,
    /// Internal symbol to English label symbol.
    pub localized: HashMap<SymbolId, SymbolId>,
}

impl Dataset {
    /// Resolves a dictionary symbol without allocation.
    #[must_use]
    pub fn symbol(&self, id: SymbolId) -> Option<&str> {
        usize::try_from(id.0)
            .ok()
            .and_then(|index| self.stored.dictionary.get(index))
            .map(String::as_str)
    }

    /// Gets a location by its typed index.
    #[must_use]
    pub fn location(&self, id: LocationId) -> Option<&LocationRecord> {
        usize::try_from(id.0)
            .ok()
            .and_then(|index| self.stored.locations.get(index))
    }

    /// Resolves an English label, falling back to the internal identifier.
    #[must_use]
    pub fn label(&self, id: SymbolId) -> Option<&str> {
        self.localized
            .get(&id)
            .and_then(|localized| self.symbol(*localized))
            .or_else(|| self.symbol(id))
    }
}

#[cfg(test)]
mod tests;
