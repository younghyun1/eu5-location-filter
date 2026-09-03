//! Selected-location detail formatting.

use slint::{Color, SharedString};

use super::AppWindow;
use super::result_model::text;
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
    .map(|symbol| dataset.label(symbol).unwrap_or("—"))
    .join("  ›  ");
    app.set_detail_breadcrumb(breadcrumb.into());
    app.set_detail_fields(fields(dataset, record).into());
}

pub(super) fn clear(app: &AppWindow) {
    app.set_detail_color(Color::from_rgb_u8(32, 40, 50));
    app.set_detail_name("Select a location".into());
    app.set_detail_key(SharedString::new());
    app.set_detail_breadcrumb(SharedString::new());
    app.set_detail_fields(SharedString::new());
}

fn fields(dataset: &Dataset, record: &LocationRecord) -> String {
    let connected = record
        .connected_sea
        .and_then(|id| dataset.location(id))
        .map_or("—", |sea| text(dataset, Some(sea.key)));
    let river = record.river.map_or_else(
        || "Missing".to_owned(),
        |value| {
            format!(
                "Level {}  •  width {:.2}  •  source {}  •  confluence {}",
                value.level.0, value.rendered_width, value.has_source, value.has_confluence
            )
        },
    );
    let harbor = record
        .harbor_suitability
        .map_or_else(|| "—".to_owned(), |value| format!("{value:.2}"));
    let movement = record.movement_assistance.map_or_else(
        || "—".to_owned(),
        |value| format!("{:.2}, {:.2}", value[0], value[1]),
    );
    format!(
        "Kind\n{}\n\nTopography\n{}\n\nVegetation\n{}\n\nClimate\n{}\n\nReligion\n{}\n\nCulture\n{}\n\nRaw material\n{}\n\nModifier\n{}\n\nCoastal\n{}\n\nConnected sea\n{}\n\nHarbor suitability\n{}\n\nMovement assistance\n{}\n\nRiver\n{}",
        record.kind.label(),
        text(dataset, Some(record.topography)),
        text(dataset, record.vegetation),
        text(dataset, record.climate),
        text(dataset, record.religion),
        text(dataset, record.culture),
        text(dataset, record.raw_material),
        text(dataset, record.modifier),
        record.coastal,
        connected,
        harbor,
        movement,
        river
    )
}
