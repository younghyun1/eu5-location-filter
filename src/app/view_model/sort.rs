//! Active sort, view-state sort tokens, and the column a sort field belongs to.

use super::column_id::ColumnId;
use crate::filter::SortField;

/// Active sort.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SortState {
    /// Sorted field.
    pub(crate) field: SortField,
    /// Ascending (`true`) or descending.
    pub(crate) ascending: bool,
}

impl Default for SortState {
    fn default() -> Self {
        Self {
            field: SortField::Name,
            ascending: true,
        }
    }
}

impl SortState {
    /// Result of a header click: the same field reverses, another field sorts ascending.
    #[must_use]
    pub(crate) fn toggled(self, field: SortField) -> Self {
        Self {
            field,
            ascending: if self.field == field {
                !self.ascending
            } else {
                true
            },
        }
    }
}

/// View-state sort token of a field; the key of the column that sorts by it.
#[must_use]
pub(crate) const fn sort_token(field: SortField) -> &'static str {
    match field {
        SortField::Color => "color",
        SortField::Name => "name",
        SortField::Identifier => "identifier",
        SortField::Kind => "kind",
        SortField::Topography => "topography",
        SortField::Vegetation => "vegetation",
        SortField::Climate => "climate",
        SortField::Continent => "continent",
        SortField::Subcontinent => "subcontinent",
        SortField::Region => "region",
        SortField::Area => "area",
        SortField::Province => "province",
        SortField::Religion => "religion",
        SortField::Culture => "culture",
        SortField::RawMaterial => "raw_material",
        SortField::Modifier => "modifier",
        SortField::Coastal => "coastal",
        SortField::RiverPresence => "river_presence",
        SortField::RiverLevel => "river_level",
        SortField::HarborSuitability => "harbor",
        SortField::MovementPresence => "movement_presence",
        SortField::MovementX => "movement_x",
        SortField::MovementY => "movement_y",
        SortField::StaticPopulationCapacity => "static_capacity",
        SortField::EquatorCapacity => "equator_capacity",
    }
}

/// Sort field for a view-state token; `rgb` is accepted as an alias of `color`.
#[must_use]
pub(crate) fn sort_from_token(token: &str) -> Option<SortField> {
    if token == "rgb" {
        return Some(SortField::Color);
    }
    SortField::ALL
        .into_iter()
        .find(|field| sort_token(*field) == token)
}

/// Column the grid's keyboard focus starts on for a sort field.
#[must_use]
pub(crate) fn column_for_sort(field: SortField) -> ColumnId {
    ColumnId::ALL
        .into_iter()
        .find(|column| column.sort_field() == field)
        .unwrap_or(ColumnId::Name)
}
