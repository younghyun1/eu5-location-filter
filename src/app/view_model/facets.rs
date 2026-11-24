//! Facet groups of the filter rail and the rail sections that hold them.

use crate::app::i18n::Text;

/// A facet group, in rail order. Replaces the string field names of the previous interface.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum FacetId {
    /// `FilterSet::kinds` group.
    Kind,
    /// `FilterSet::continent` group.
    Continent,
    /// `FilterSet::subcontinent` group.
    Subcontinent,
    /// `FilterSet::region` group.
    Region,
    /// `FilterSet::area` group.
    Area,
    /// `FilterSet::province` group.
    Province,
    /// `FilterSet::topography` group.
    Topography,
    /// `FilterSet::vegetation` group.
    Vegetation,
    /// `FilterSet::climate` group.
    Climate,
    /// `FilterSet::religion` group.
    Religion,
    /// `FilterSet::culture` group.
    Culture,
    /// `FilterSet::raw_material` group.
    RawMaterial,
    /// `FilterSet::modifier` group.
    Modifier,
    /// `FilterSet::coastal` group.
    Coastal,
    /// `FilterSet::river_presence` group.
    RiverPresence,
    /// `FilterSet::river_level_min` group.
    RiverLevelMin,
    /// `FilterSet::river_level_max` group.
    RiverLevelMax,
    /// `FilterSet::harbor_presence` group.
    HarborPresence,
    /// `FilterSet::movement_presence` group.
    MovementPresence,
}

impl FacetId {
    /// Number of groups.
    pub(crate) const COUNT: usize = 19;

    /// Every group in rail order.
    pub(crate) const ALL: [Self; Self::COUNT] = [
        Self::Kind,
        Self::Continent,
        Self::Subcontinent,
        Self::Region,
        Self::Area,
        Self::Province,
        Self::Topography,
        Self::Vegetation,
        Self::Climate,
        Self::Religion,
        Self::Culture,
        Self::RawMaterial,
        Self::Modifier,
        Self::Coastal,
        Self::RiverPresence,
        Self::RiverLevelMin,
        Self::RiverLevelMax,
        Self::HarborPresence,
        Self::MovementPresence,
    ];

    /// Position in `ALL`, used to index per-group arrays.
    #[must_use]
    pub(crate) const fn index(self) -> usize {
        self as usize
    }

    /// Stable lowercase key used in widget identifiers.
    #[must_use]
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::Kind => "kind",
            Self::Continent => "continent",
            Self::Subcontinent => "subcontinent",
            Self::Region => "region",
            Self::Area => "area",
            Self::Province => "province",
            Self::Topography => "topography",
            Self::Vegetation => "vegetation",
            Self::Climate => "climate",
            Self::Religion => "religion",
            Self::Culture => "culture",
            Self::RawMaterial => "raw_material",
            Self::Modifier => "modifier",
            Self::Coastal => "coastal",
            Self::RiverPresence => "river_presence",
            Self::RiverLevelMin => "river_level_min",
            Self::RiverLevelMax => "river_level_max",
            Self::HarborPresence => "harbor_presence",
            Self::MovementPresence => "movement_presence",
        }
    }

    /// Group title.
    #[must_use]
    pub(crate) const fn title(self) -> Text {
        match self {
            Self::Kind => Text::FacetKind,
            Self::Continent => Text::FacetContinent,
            Self::Subcontinent => Text::FacetSubcontinent,
            Self::Region => Text::FacetRegion,
            Self::Area => Text::FacetArea,
            Self::Province => Text::FacetProvince,
            Self::Topography => Text::FacetTopography,
            Self::Vegetation => Text::FacetVegetation,
            Self::Climate => Text::FacetClimate,
            Self::Religion => Text::FacetReligion,
            Self::Culture => Text::FacetCulture,
            Self::RawMaterial => Text::FacetRawMaterial,
            Self::Modifier => Text::FacetModifier,
            Self::Coastal => Text::FacetCoastal,
            Self::RiverPresence => Text::FacetRiverPresence,
            Self::RiverLevelMin => Text::FacetRiverLevelMin,
            Self::RiverLevelMax => Text::FacetRiverLevelMax,
            Self::HarborPresence => Text::FacetHarborPresence,
            Self::MovementPresence => Text::FacetMovementPresence,
        }
    }

    /// Rail section holding the group.
    #[must_use]
    pub(crate) const fn section(self) -> RailSection {
        match self {
            Self::Kind => RailSection::Location,
            Self::Continent | Self::Subcontinent | Self::Region | Self::Area | Self::Province => {
                RailSection::Geography
            }
            Self::Topography | Self::Vegetation | Self::Climate => RailSection::Terrain,
            Self::Religion | Self::Culture => RailSection::Society,
            Self::RawMaterial | Self::Modifier => RailSection::Economy,
            Self::Coastal
            | Self::RiverPresence
            | Self::RiverLevelMin
            | Self::RiverLevelMax
            | Self::HarborPresence
            | Self::MovementPresence => RailSection::MapData,
        }
    }

    /// Whether the group starts folded; only Type starts expanded.
    #[must_use]
    pub(crate) const fn starts_folded(self) -> bool {
        !matches!(self, Self::Kind)
    }

    /// Whether the option list starts with the missing-value option.
    #[must_use]
    pub(crate) const fn has_missing_option(self) -> bool {
        matches!(
            self,
            Self::Vegetation
                | Self::Climate
                | Self::Religion
                | Self::Culture
                | Self::RawMaterial
                | Self::Modifier
        )
    }
}

/// Rail section, in rail order.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum RailSection {
    /// `rail.section.location`.
    Location,
    /// `rail.section.geography`.
    Geography,
    /// `rail.section.terrain`.
    Terrain,
    /// `rail.section.society`.
    Society,
    /// `rail.section.economy`.
    Economy,
    /// `rail.section.map_data`.
    MapData,
}

impl RailSection {
    /// Every section in rail order.
    pub(crate) const ALL: [Self; 6] = [
        Self::Location,
        Self::Geography,
        Self::Terrain,
        Self::Society,
        Self::Economy,
        Self::MapData,
    ];

    /// Section title.
    #[must_use]
    pub(crate) const fn title(self) -> Text {
        match self {
            Self::Location => Text::RailSectionLocation,
            Self::Geography => Text::RailSectionGeography,
            Self::Terrain => Text::RailSectionTerrain,
            Self::Society => Text::RailSectionSociety,
            Self::Economy => Text::RailSectionEconomy,
            Self::MapData => Text::RailSectionMapData,
        }
    }
}
