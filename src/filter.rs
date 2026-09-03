//! Allocation-bounded filtering and stable sorting over compact records.

use std::cmp::Ordering;
use std::collections::HashSet;
use std::sync::Arc;

use crate::AppError;
use crate::model::{Dataset, LocationId, LocationKind, LocationRecord, MapColor, SymbolId};

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
    fn matches(self, value: f32) -> bool {
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

/// Precomputed search strings and reusable index scanning logic.
pub struct FilterEngine {
    dataset: Arc<Dataset>,
    searchable: Vec<String>,
}

impl FilterEngine {
    /// Precomputes Unicode-lowercased name and identifier strings once at startup.
    #[must_use]
    pub fn new(dataset: Arc<Dataset>) -> Self {
        let searchable = dataset
            .stored
            .locations
            .iter()
            .map(|record| {
                let name = dataset.symbol(record.name).unwrap_or_default();
                let key = dataset.symbol(record.key).unwrap_or_default();
                format!("{}\0{}", name.to_lowercase(), key.to_lowercase())
            })
            .collect();
        Self {
            dataset,
            searchable,
        }
    }

    /// Scans compact records and returns only matching location IDs.
    #[must_use]
    pub fn apply(&self, filters: &FilterSet, sort: SortField, ascending: bool) -> Vec<LocationId> {
        let query = filters.search.to_lowercase();
        let mut ids: Vec<LocationId> = self
            .dataset
            .stored
            .locations
            .iter()
            .filter(|record| self.matches(record, filters, &query))
            .map(|record| record.id)
            .collect();
        ids.sort_by(|left, right| self.compare(*left, *right, sort, ascending));
        ids
    }

    /// Preserves selection only while its ID remains in the filtered index.
    #[must_use]
    pub fn preserve_selection(
        selected: Option<LocationId>,
        visible: &[LocationId],
    ) -> Option<LocationId> {
        selected.filter(|id| visible.contains(id))
    }

    fn matches(&self, record: &LocationRecord, filters: &FilterSet, query: &str) -> bool {
        let searchable = usize::try_from(record.id.0)
            .ok()
            .and_then(|index| self.searchable.get(index))
            .map(String::as_str)
            .unwrap_or_default();
        (filters.show_impassable || record.kind != LocationKind::Impassable)
            && (query.is_empty() || searchable.contains(query))
            && (filters.kinds.is_empty() || filters.kinds.contains(&record.kind))
            && (filters.topographies.is_empty()
                || filters.topographies.contains(&record.topography))
            && (filters.vegetation.is_empty() || filters.vegetation.contains(&record.vegetation))
            && (filters.climates.is_empty() || filters.climates.contains(&record.climate))
            && facet_matches(filters.continent, Some(record.hierarchy.continent))
            && facet_matches(filters.subcontinent, Some(record.hierarchy.subcontinent))
            && facet_matches(filters.region, Some(record.hierarchy.region))
            && facet_matches(filters.area, Some(record.hierarchy.area))
            && facet_matches(filters.province, Some(record.hierarchy.province))
            && facet_matches(filters.religion, record.religion)
            && facet_matches(filters.culture, record.culture)
            && facet_matches(filters.raw_material, record.raw_material)
            && facet_matches(filters.modifier, record.modifier)
            && filters.rgb.is_none_or(|color| record.color == color)
            && filters.coastal.is_none_or(|value| record.coastal == value)
            && filters
                .river_presence
                .is_none_or(|value| record.river.is_some() == value)
            && record.river.as_ref().is_none_or(|river| {
                filters
                    .river_level_min
                    .is_none_or(|minimum| river.level.0 >= minimum)
                    && filters
                        .river_level_max
                        .is_none_or(|maximum| river.level.0 <= maximum)
            })
            && numeric_matches(
                filters.harbor_presence,
                filters.harbor_range,
                record.harbor_suitability,
            )
            && filters
                .movement_presence
                .is_none_or(|value| record.movement_assistance.is_some() == value)
            && record.movement_assistance.is_none_or(|value| {
                filters.movement_x.matches(value[0]) && filters.movement_y.matches(value[1])
            })
    }

    fn compare(
        &self,
        left: LocationId,
        right: LocationId,
        field: SortField,
        ascending: bool,
    ) -> Ordering {
        let Some(left_record) = self.dataset.location(left) else {
            return Ordering::Equal;
        };
        let Some(right_record) = self.dataset.location(right) else {
            return Ordering::Equal;
        };
        let ordering = match field {
            SortField::Name => symbols(
                &self.dataset,
                Some(left_record.name),
                Some(right_record.name),
            ),
            SortField::Identifier => {
                symbols(&self.dataset, Some(left_record.key), Some(right_record.key))
            }
            SortField::Kind => left_record.kind.cmp(&right_record.kind),
            SortField::Topography => symbols(
                &self.dataset,
                Some(left_record.topography),
                Some(right_record.topography),
            ),
            SortField::Vegetation => optional_symbols(
                &self.dataset,
                left_record.vegetation,
                right_record.vegetation,
            ),
            SortField::Climate => {
                optional_symbols(&self.dataset, left_record.climate, right_record.climate)
            }
            SortField::RiverLevel => optional_values(
                left_record.river.as_ref().map(|value| value.level.0),
                right_record.river.as_ref().map(|value| value.level.0),
            ),
        };
        let directed = if ascending {
            ordering
        } else {
            reverse_non_null(ordering, field, left_record, right_record)
        };
        directed.then_with(|| left.cmp(&right))
    }
}

fn facet_matches(filter: OptionalFacet, value: Option<SymbolId>) -> bool {
    match filter {
        OptionalFacet::Any => true,
        OptionalFacet::Missing => value.is_none(),
        OptionalFacet::Value(expected) => value == Some(expected),
    }
}

fn numeric_matches(filter: OptionalNumeric, range: FloatRange, value: Option<f32>) -> bool {
    match (filter, value) {
        (OptionalNumeric::Any, None) | (OptionalNumeric::Missing, None) => true,
        (OptionalNumeric::Present, None) | (OptionalNumeric::Missing, Some(_)) => false,
        (OptionalNumeric::Any | OptionalNumeric::Present, Some(value)) => range.matches(value),
    }
}

fn symbols(dataset: &Dataset, left: Option<SymbolId>, right: Option<SymbolId>) -> Ordering {
    optional_values(
        left.and_then(|value| dataset.symbol(value))
            .map(str::to_lowercase),
        right
            .and_then(|value| dataset.symbol(value))
            .map(str::to_lowercase),
    )
}

fn optional_symbols(
    dataset: &Dataset,
    left: Option<SymbolId>,
    right: Option<SymbolId>,
) -> Ordering {
    symbols(dataset, left, right)
}

fn optional_values<T: Ord>(left: Option<T>, right: Option<T>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn reverse_non_null(
    ordering: Ordering,
    field: SortField,
    left: &LocationRecord,
    right: &LocationRecord,
) -> Ordering {
    let nullable = match field {
        SortField::Vegetation => (left.vegetation.is_none(), right.vegetation.is_none()),
        SortField::Climate => (left.climate.is_none(), right.climate.is_none()),
        SortField::RiverLevel => (left.river.is_none(), right.river.is_none()),
        _ => (false, false),
    };
    if nullable.0 != nullable.1 {
        ordering
    } else {
        ordering.reverse()
    }
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

#[cfg(test)]
mod tests;
