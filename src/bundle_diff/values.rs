//! Rendered field values of one record, in a fixed order shared by both bundles.

use crate::model::{Dataset, LocationRecord, SymbolId};

fn text(dataset: &Dataset, symbol: Option<SymbolId>) -> String {
    match symbol.and_then(|id| dataset.symbol(id)) {
        Some(value) => value.to_owned(),
        None => "-".to_owned(),
    }
}

/// Field names and rendered values in a fixed order shared by both sides.
pub(super) fn fields(dataset: &Dataset, record: &LocationRecord) -> Vec<(&'static str, String)> {
    let connected = match record
        .connected_sea
        .and_then(|id| dataset.location(id))
        .and_then(|sea| dataset.symbol(sea.key))
    {
        Some(key) => key.to_owned(),
        None => "-".to_owned(),
    };
    let capacity = record.static_population_capacity;
    vec![
        ("area", text(dataset, Some(record.hierarchy.area))),
        (
            "capacity_equator",
            capacity.map_or_else(|| "-".to_owned(), |value| value.equator.0.to_string()),
        ),
        (
            "capacity_modifier_bp",
            capacity.map_or_else(
                || "-".to_owned(),
                |value| value.modifier_basis_points.to_string(),
            ),
        ),
        (
            "capacity_total",
            capacity.map_or_else(|| "-".to_owned(), |value| value.total.0.to_string()),
        ),
        (
            "capacity_vegetation",
            capacity.map_or_else(|| "-".to_owned(), |value| value.vegetation.0.to_string()),
        ),
        ("climate", text(dataset, record.climate)),
        ("coastal", record.coastal.to_string()),
        ("color", record.color.hex()),
        ("connected_sea", connected),
        ("continent", text(dataset, Some(record.hierarchy.continent))),
        ("culture", text(dataset, record.culture)),
        (
            "harbor_suitability",
            record
                .harbor_suitability
                .map_or_else(|| "-".to_owned(), |value| format!("{value:.4}")),
        ),
        ("kind", record.kind.label().to_owned()),
        ("modifier", text(dataset, record.modifier)),
        (
            "movement_assistance",
            record.movement_assistance.map_or_else(
                || "-".to_owned(),
                |value| format!("{:.4},{:.4}", value[0], value[1]),
            ),
        ),
        ("name", text(dataset, Some(record.name))),
        ("province", text(dataset, Some(record.hierarchy.province))),
        ("raw_material", text(dataset, record.raw_material)),
        ("region", text(dataset, Some(record.hierarchy.region))),
        ("religion", text(dataset, record.religion)),
        (
            "river",
            record.river.map_or_else(
                || "-".to_owned(),
                |river| {
                    format!(
                        "L{} source={} confluence={}",
                        river.level.0, river.has_source, river.has_confluence
                    )
                },
            ),
        ),
        (
            "subcontinent",
            text(dataset, Some(record.hierarchy.subcontinent)),
        ),
        ("topography", text(dataset, Some(record.topography))),
        ("vegetation", text(dataset, record.vegetation)),
    ]
}

