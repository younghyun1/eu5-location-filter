//! Result column identity and the per-column defaults from the interface contract.

use super::row::{CellAlign, CellFont};
use crate::app::i18n::Text;
use crate::filter::SortField;

/// A result column, in default display order, which is also the column menu order.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum ColumnId {
    /// `color`.
    Color,
    /// `name`.
    Name,
    /// `identifier`.
    Identifier,
    /// `kind`.
    Kind,
    /// `topography`.
    Topography,
    /// `vegetation`.
    Vegetation,
    /// `climate`.
    Climate,
    /// `continent`.
    Continent,
    /// `subcontinent`.
    Subcontinent,
    /// `region`.
    Region,
    /// `area`.
    Area,
    /// `province`.
    Province,
    /// `religion`.
    Religion,
    /// `culture`.
    Culture,
    /// `raw_material`.
    RawMaterial,
    /// `modifier`.
    Modifier,
    /// `rgb`.
    Rgb,
    /// `coastal`.
    Coastal,
    /// `river_presence`.
    RiverPresence,
    /// `river_level`.
    RiverLevel,
    /// `harbor`.
    Harbor,
    /// `movement_presence`.
    MovementPresence,
    /// `movement_x`.
    MovementX,
    /// `movement_y`.
    MovementY,
    /// `static_capacity`.
    StaticCapacity,
    /// `equator_capacity`.
    EquatorCapacity,
}

impl ColumnId {
    /// Number of columns.
    pub(crate) const COUNT: usize = 26;

    /// Every column in display order.
    pub(crate) const ALL: [Self; Self::COUNT] = [
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
        Self::Rgb,
        Self::Coastal,
        Self::RiverPresence,
        Self::RiverLevel,
        Self::Harbor,
        Self::MovementPresence,
        Self::MovementX,
        Self::MovementY,
        Self::StaticCapacity,
        Self::EquatorCapacity,
    ];

    /// Position in `ALL`.
    #[must_use]
    pub(crate) const fn index(self) -> usize {
        self as usize
    }

    /// Stable key, shared with sort tokens in the view-state string.
    #[cfg(test)]
    #[must_use]
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::Color => "color",
            Self::Name => "name",
            Self::Identifier => "identifier",
            Self::Kind => "kind",
            Self::Topography => "topography",
            Self::Vegetation => "vegetation",
            Self::Climate => "climate",
            Self::Continent => "continent",
            Self::Subcontinent => "subcontinent",
            Self::Region => "region",
            Self::Area => "area",
            Self::Province => "province",
            Self::Religion => "religion",
            Self::Culture => "culture",
            Self::RawMaterial => "raw_material",
            Self::Modifier => "modifier",
            Self::Rgb => "rgb",
            Self::Coastal => "coastal",
            Self::RiverPresence => "river_presence",
            Self::RiverLevel => "river_level",
            Self::Harbor => "harbor",
            Self::MovementPresence => "movement_presence",
            Self::MovementX => "movement_x",
            Self::MovementY => "movement_y",
            Self::StaticCapacity => "static_capacity",
            Self::EquatorCapacity => "equator_capacity",
        }
    }

    /// Column for a stable key.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|column| column.key() == key)
    }

    /// Header and column menu label.
    #[must_use]
    pub(crate) const fn title(self) -> Text {
        match self {
            Self::Color => Text::ColumnColor,
            Self::Name => Text::ColumnName,
            Self::Identifier => Text::ColumnIdentifier,
            Self::Kind => Text::ColumnKind,
            Self::Topography => Text::ColumnTopography,
            Self::Vegetation => Text::ColumnVegetation,
            Self::Climate => Text::ColumnClimate,
            Self::Continent => Text::ColumnContinent,
            Self::Subcontinent => Text::ColumnSubcontinent,
            Self::Region => Text::ColumnRegion,
            Self::Area => Text::ColumnArea,
            Self::Province => Text::ColumnProvince,
            Self::Religion => Text::ColumnReligion,
            Self::Culture => Text::ColumnCulture,
            Self::RawMaterial => Text::ColumnRawMaterial,
            Self::Modifier => Text::ColumnModifier,
            Self::Rgb => Text::ColumnRgb,
            Self::Coastal => Text::ColumnCoastal,
            Self::RiverPresence => Text::ColumnRiverPresence,
            Self::RiverLevel => Text::ColumnRiverLevel,
            Self::Harbor => Text::ColumnHarbor,
            Self::MovementPresence => Text::ColumnMovementPresence,
            Self::MovementX => Text::ColumnMovementX,
            Self::MovementY => Text::ColumnMovementY,
            Self::StaticCapacity => Text::ColumnStaticCapacity,
            Self::EquatorCapacity => Text::ColumnEquatorCapacity,
        }
    }

    /// Width at first start, in logical pixels.
    #[must_use]
    pub(crate) const fn default_width(self) -> f32 {
        match self {
            Self::Color => 108.0,
            Self::Name => 210.0,
            Self::Identifier => 150.0,
            Self::Kind => 76.0,
            Self::Topography => 110.0,
            Self::Vegetation => 110.0,
            Self::Climate => 100.0,
            Self::Continent => 130.0,
            Self::Subcontinent => 140.0,
            Self::Region => 150.0,
            Self::Area => 150.0,
            Self::Province => 160.0,
            Self::Religion => 130.0,
            Self::Culture => 130.0,
            Self::RawMaterial => 150.0,
            Self::Modifier => 150.0,
            Self::Rgb => 150.0,
            Self::Coastal => 82.0,
            Self::RiverPresence => 90.0,
            Self::RiverLevel => 112.0,
            Self::Harbor => 84.0,
            Self::MovementPresence => 100.0,
            Self::MovementX => 104.0,
            Self::MovementY => 104.0,
            Self::StaticCapacity => 126.0,
            Self::EquatorCapacity => 130.0,
        }
    }

    /// Visibility at first start.
    #[must_use]
    pub(crate) const fn default_visible(self) -> bool {
        matches!(
            self,
            Self::Color
                | Self::Name
                | Self::Identifier
                | Self::Kind
                | Self::Topography
                | Self::Vegetation
                | Self::Climate
                | Self::RawMaterial
                | Self::RiverLevel
                | Self::Harbor
                | Self::StaticCapacity
        )
    }

    /// Logical alignment of cells and header label.
    #[must_use]
    pub(crate) const fn align(self) -> CellAlign {
        match self {
            Self::RiverLevel
            | Self::Harbor
            | Self::MovementX
            | Self::MovementY
            | Self::StaticCapacity
            | Self::EquatorCapacity => CellAlign::End,
            _ => CellAlign::Start,
        }
    }

    /// Cell drawing style.
    #[must_use]
    pub(crate) const fn font(self) -> CellFont {
        match self {
            Self::Color => CellFont::Swatch,
            Self::Name => CellFont::SansMedium,
            Self::Identifier
            | Self::Rgb
            | Self::RiverLevel
            | Self::Harbor
            | Self::MovementX
            | Self::MovementY
            | Self::StaticCapacity
            | Self::EquatorCapacity => CellFont::Mono,
            _ => CellFont::Sans,
        }
    }

    /// Sort field a header click selects.
    #[must_use]
    pub(crate) const fn sort_field(self) -> SortField {
        match self {
            Self::Color => SortField::Color,
            Self::Name => SortField::Name,
            Self::Identifier => SortField::Identifier,
            Self::Kind => SortField::Kind,
            Self::Topography => SortField::Topography,
            Self::Vegetation => SortField::Vegetation,
            Self::Climate => SortField::Climate,
            Self::Continent => SortField::Continent,
            Self::Subcontinent => SortField::Subcontinent,
            Self::Region => SortField::Region,
            Self::Area => SortField::Area,
            Self::Province => SortField::Province,
            Self::Religion => SortField::Religion,
            Self::Culture => SortField::Culture,
            Self::RawMaterial => SortField::RawMaterial,
            Self::Modifier => SortField::Modifier,
            Self::Rgb => SortField::Color,
            Self::Coastal => SortField::Coastal,
            Self::RiverPresence => SortField::RiverPresence,
            Self::RiverLevel => SortField::RiverLevel,
            Self::Harbor => SortField::HarborSuitability,
            Self::MovementPresence => SortField::MovementPresence,
            Self::MovementX => SortField::MovementX,
            Self::MovementY => SortField::MovementY,
            Self::StaticCapacity => SortField::StaticPopulationCapacity,
            Self::EquatorCapacity => SortField::EquatorCapacity,
        }
    }
}
