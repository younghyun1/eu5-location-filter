//! Selected-location detail formatting.

use slint::{Color, ModelRc, SharedString};

use super::result_model::{format_population, text};
use super::{AppWindow, DetailField};
use crate::model::{Dataset, LocationId, LocationRecord};

pub(super) fn show(app: &AppWindow, dataset: &Dataset, id: LocationId) {
    let Some(record) = dataset.location(id) else {
        return;
    };
    let [red, green, blue] = record.color.components();
    app.set_detail_color(Color::from_rgb_u8(red, green, blue));
    app.set_detail_name(text(dataset, Some(record.name)).into());
    app.set_detail_key(text(dataset, Some(record.key)).into());
    app.set_detail_hex(record.color.hex().into());
    app.set_detail_rgb(format!("rgb({red}, {green}, {blue})").into());
    let breadcrumb = [
        record.hierarchy.continent,
        record.hierarchy.subcontinent,
        record.hierarchy.region,
        record.hierarchy.area,
        record.hierarchy.province,
    ]
    .map(|symbol| dataset.label(symbol).unwrap_or("-"))
    .join("  >  ");
    app.set_detail_breadcrumb(breadcrumb.into());
    let fields = fields(dataset, record);
    app.set_detail_fields(ModelRc::from(fields.as_slice()));
}

pub(super) fn clear(app: &AppWindow) {
    app.set_detail_color(Color::from_rgb_u8(32, 40, 50));
    app.set_detail_name("Select a location".into());
    app.set_detail_key(SharedString::new());
    app.set_detail_breadcrumb(SharedString::new());
    app.set_detail_fields(ModelRc::default());
}

fn fields(dataset: &Dataset, record: &LocationRecord) -> Vec<DetailField> {
    let connected = record
        .connected_sea
        .and_then(|id| dataset.location(id))
        .map_or("-", |sea| text(dataset, Some(sea.key)));
    let river = record.river.map_or_else(
        || "Missing".to_owned(),
        |value| {
            format!(
                "{}: level {} (+{}% capacity), source {}, confluence {}",
                value.level.label(),
                value.level.0,
                value.level.0 * 10,
                yes_no(value.has_source),
                yes_no(value.has_confluence)
            )
        },
    );
    let harbor = record
        .harbor_suitability
        .map_or_else(|| "-".to_owned(), |value| format!("{value:.2}"));
    let movement = record.movement_assistance.map_or_else(
        || "-".to_owned(),
        |value| format!("{:.2}, {:.2}", value[0], value[1]),
    );
    let capacity = record.static_population_capacity;
    let total_capacity = capacity
        .map(|value| format_population(value.total.0))
        .unwrap_or_else(|| "-".to_owned());
    let vegetation_capacity = capacity
        .map(|value| format_population(value.vegetation.0))
        .unwrap_or_else(|| "-".to_owned());
    let equator_capacity = capacity
        .map(|value| format_population(value.equator.0))
        .unwrap_or_else(|| "-".to_owned());
    let capacity_modifier = capacity.map_or_else(
        || "-".to_owned(),
        |value| format!("{:+.2}%", f32::from(value.modifier_basis_points) / 100.0),
    );
    let raw_material = record.raw_material.map_or_else(
        || "-".to_owned(),
        |symbol| {
            let key = dataset.symbol(symbol).unwrap_or("-");
            dataset.label(symbol).unwrap_or(key).to_owned()
        },
    );
    vec![
        field("Type", record.kind.label()),
        field("Topography", text(dataset, Some(record.topography))),
        field("Vegetation", text(dataset, record.vegetation)),
        field("Climate", text(dataset, record.climate)),
        field("Religion", text(dataset, record.religion)),
        field("Culture", text(dataset, record.culture)),
        field("Raw material", &raw_material),
        field("Modifier", text(dataset, record.modifier)),
        field("Coastal", if record.coastal { "Yes" } else { "No" }),
        field("Connected sea", connected),
        field("Harbor suitability", &harbor),
        field("Movement assistance", &movement),
        field("River", &river),
        field("Static population capacity", &total_capacity),
        field("Vegetation capacity", &vegetation_capacity),
        field("Closeness to equator", &equator_capacity),
        field("Static capacity modifier", &capacity_modifier),
    ]
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn field(label: &str, value: &str) -> DetailField {
    DetailField {
        label: label.into(),
        value: value.into(),
    }
}
