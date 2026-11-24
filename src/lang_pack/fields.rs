//! Which record fields carry localized labels and which result columns sort by them.

use std::borrow::Cow;

use crate::filter::SortField;
use crate::model::{LocationRecord, SymbolId};

/// Record fields whose symbol is shown through a localized label.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LabelField {
    /// `LocationRecord.topography`.
    Topography,
    /// `LocationRecord.vegetation`.
    Vegetation,
    /// `LocationRecord.climate`.
    Climate,
    /// `Hierarchy.continent`.
    Continent,
    /// `Hierarchy.subcontinent`.
    Subcontinent,
    /// `Hierarchy.region`.
    Region,
    /// `Hierarchy.area`.
    Area,
    /// `Hierarchy.province`.
    Province,
    /// `LocationRecord.religion`.
    Religion,
    /// `LocationRecord.culture`.
    Culture,
    /// `LocationRecord.raw_material`.
    RawMaterial,
    /// `LocationRecord.modifier`.
    Modifier,
}

impl LabelField {
    /// Every labelled field in [`SortField::ALL`] order.
    pub const ALL: [Self; 12] = [
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
    ];

    /// Stable lowercase name used in diff reports and view-state strings.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
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
        }
    }

    /// Result column sorted by this field's label.
    #[must_use]
    pub const fn sort_field(self) -> SortField {
        match self {
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
        }
    }

    /// Symbol this field holds on one record, if any.
    #[must_use]
    pub const fn symbol(self, record: &LocationRecord) -> Option<SymbolId> {
        match self {
            Self::Topography => Some(record.topography),
            Self::Vegetation => record.vegetation,
            Self::Climate => record.climate,
            Self::Continent => Some(record.hierarchy.continent),
            Self::Subcontinent => Some(record.hierarchy.subcontinent),
            Self::Region => Some(record.hierarchy.region),
            Self::Area => Some(record.hierarchy.area),
            Self::Province => Some(record.hierarchy.province),
            Self::Religion => record.religion,
            Self::Culture => record.culture,
            Self::RawMaterial => record.raw_material,
            Self::Modifier => record.modifier,
        }
    }

    /// Localization key that carries the label for an internal identifier.
    ///
    /// Static modifiers are the one field not keyed by the identifier itself: every language
    /// names them `STATIC_MODIFIER_NAME_<key>`.
    #[must_use]
    pub fn localization_key(self, key: &str) -> Cow<'_, str> {
        match self {
            Self::Modifier => Cow::Owned(format!("STATIC_MODIFIER_NAME_{key}")),
            _ => Cow::Borrowed(key),
        }
    }
}

/// Columns sorted by a localized string; their orders live in a language section.
pub const LABEL_SORT_FIELDS: [SortField; 13] = [
    SortField::Name,
    SortField::Topography,
    SortField::Vegetation,
    SortField::Climate,
    SortField::Continent,
    SortField::Subcontinent,
    SortField::Region,
    SortField::Area,
    SortField::Province,
    SortField::Religion,
    SortField::Culture,
    SortField::RawMaterial,
    SortField::Modifier,
];

/// Columns whose order never depends on language; their orders stay in the filter index.
pub const LANGUAGE_INDEPENDENT_SORT_FIELDS: [SortField; 12] = [
    SortField::Color,
    SortField::Identifier,
    SortField::Kind,
    SortField::Coastal,
    SortField::RiverPresence,
    SortField::RiverLevel,
    SortField::HarborSuitability,
    SortField::MovementPresence,
    SortField::MovementX,
    SortField::MovementY,
    SortField::StaticPopulationCapacity,
    SortField::EquatorCapacity,
];

/// Returns whether a column's order comes from the active language section.
#[must_use]
pub const fn is_label_dependent(field: SortField) -> bool {
    matches!(
        field,
        SortField::Name
            | SortField::Topography
            | SortField::Vegetation
            | SortField::Climate
            | SortField::Continent
            | SortField::Subcontinent
            | SortField::Region
            | SortField::Area
            | SortField::Province
            | SortField::Religion
            | SortField::Culture
            | SortField::RawMaterial
            | SortField::Modifier
    )
}

#[cfg(test)]
mod tests {
    use super::{LABEL_SORT_FIELDS, LANGUAGE_INDEPENDENT_SORT_FIELDS, is_label_dependent};
    use crate::filter::SortField;

    #[test]
    fn sort_fields_partition_every_column() {
        let total = LABEL_SORT_FIELDS.len() + LANGUAGE_INDEPENDENT_SORT_FIELDS.len();
        assert_eq!(total, SortField::ALL.len());
        for field in SortField::ALL {
            let labelled = LABEL_SORT_FIELDS.contains(&field);
            let independent = LANGUAGE_INDEPENDENT_SORT_FIELDS.contains(&field);
            assert_ne!(labelled, independent);
            assert_eq!(is_label_dependent(field), labelled);
        }
    }
}
