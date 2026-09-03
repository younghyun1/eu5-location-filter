//! Bounded result-column configuration shared by the table and column menu.

use std::rc::Rc;

use slint::{Model, SharedString, VecModel};

use super::ColumnSpec;
use crate::filter::SortField;

const HANDLE_WIDTH: f32 = 6.0;
const TABLE_PADDING: f32 = 20.0;

pub(super) fn initial() -> Rc<VecModel<ColumnSpec>> {
    let columns = [
        spec("color", "Color", 108.0, true),
        spec("name", "Name", 210.0, true),
        spec("identifier", "Identifier", 150.0, true),
        spec("kind", "Kind", 76.0, true),
        spec("topography", "Topography", 110.0, true),
        spec("vegetation", "Vegetation", 110.0, true),
        spec("climate", "Climate", 100.0, true),
        spec("continent", "Continent", 130.0, false),
        spec("subcontinent", "Subcontinent", 140.0, false),
        spec("region", "Region", 150.0, false),
        spec("area", "Area", 150.0, false),
        spec("province", "Province", 160.0, false),
        spec("religion", "Religion", 130.0, false),
        spec("culture", "Culture", 130.0, false),
        spec("raw_material", "Raw material", 130.0, false),
        spec("modifier", "Modifier", 150.0, false),
        spec("rgb", "RGB", 150.0, false),
        spec("coastal", "Coastal", 82.0, false),
        spec("river_presence", "River", 90.0, false),
        spec("river_level", "River level", 96.0, true),
        spec("harbor", "Harbor", 84.0, true),
        spec("movement_presence", "Movement", 100.0, false),
        spec("movement_x", "Movement X", 104.0, false),
        spec("movement_y", "Movement Y", 104.0, false),
    ];
    Rc::new(VecModel::from(columns.to_vec()))
}

pub(super) fn set_visible(model: &VecModel<ColumnSpec>, key: &str, visible: bool) {
    if !visible && visible_count(model) <= 1 {
        return;
    }
    update(model, key, |column| column.visible = visible);
}

pub(super) fn set_width(model: &VecModel<ColumnSpec>, key: &str, width: f32) {
    update(model, key, |column| column.width = width.clamp(54.0, 460.0));
}

pub(super) fn total_width(model: &VecModel<ColumnSpec>) -> f32 {
    TABLE_PADDING
        + (0..model.row_count())
            .filter_map(|index| model.row_data(index))
            .filter(|column| column.visible)
            .map(|column| column.width + HANDLE_WIDTH)
            .sum::<f32>()
}

pub(super) fn sort_field(key: &str) -> Option<SortField> {
    Some(match key {
        "color" | "rgb" => SortField::Color,
        "name" => SortField::Name,
        "identifier" => SortField::Identifier,
        "kind" => SortField::Kind,
        "topography" => SortField::Topography,
        "vegetation" => SortField::Vegetation,
        "climate" => SortField::Climate,
        "continent" => SortField::Continent,
        "subcontinent" => SortField::Subcontinent,
        "region" => SortField::Region,
        "area" => SortField::Area,
        "province" => SortField::Province,
        "religion" => SortField::Religion,
        "culture" => SortField::Culture,
        "raw_material" => SortField::RawMaterial,
        "modifier" => SortField::Modifier,
        "coastal" => SortField::Coastal,
        "river_presence" => SortField::RiverPresence,
        "river_level" => SortField::RiverLevel,
        "harbor" => SortField::HarborSuitability,
        "movement_presence" => SortField::MovementPresence,
        "movement_x" => SortField::MovementX,
        "movement_y" => SortField::MovementY,
        _ => return None,
    })
}

fn spec(key: &str, label: &str, width: f32, visible: bool) -> ColumnSpec {
    ColumnSpec {
        key: SharedString::from(key),
        label: SharedString::from(label),
        width,
        visible,
    }
}

fn update(model: &VecModel<ColumnSpec>, key: &str, change: impl FnOnce(&mut ColumnSpec)) {
    for index in 0..model.row_count() {
        let Some(mut column) = model.row_data(index) else {
            continue;
        };
        if column.key == key {
            change(&mut column);
            model.set_row_data(index, column);
            return;
        }
    }
}

fn visible_count(model: &VecModel<ColumnSpec>) -> usize {
    (0..model.row_count())
        .filter_map(|index| model.row_data(index))
        .filter(|column| column.visible)
        .count()
}

#[cfg(test)]
mod tests {
    use super::{initial, set_visible, set_width, total_width};
    use slint::Model;

    #[test]
    fn column_updates_are_bounded_and_recompute_width() {
        let columns = initial();
        let before = total_width(&columns);
        set_visible(&columns, "continent", true);
        assert!(total_width(&columns) > before);
        set_width(&columns, "name", 400.0);
        let name = (0..columns.row_count())
            .filter_map(|index| columns.row_data(index))
            .find(|column| column.key == "name");
        assert_eq!(name.map(|column| column.width), Some(400.0));
    }
}
