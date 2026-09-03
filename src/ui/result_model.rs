//! Virtualized Slint model backed by filtered IDs and shared dictionary strings.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use slint::{Color, Model, ModelNotify, ModelTracker, SharedString};

use super::LocationRow;
use crate::AppError;
use crate::model::{
    Dataset, LocationId, LocationRecord, SymbolId, is_food_producing, is_gold_or_silver,
};

/// The only mutable result state is the compact vector of visible IDs.
pub(super) struct ResultModel {
    ids: RefCell<Vec<LocationId>>,
    dataset: Arc<Dataset>,
    symbols: Vec<SharedString>,
    labels: Vec<SharedString>,
    notify: ModelNotify,
}

impl ResultModel {
    pub(super) fn new(dataset: &Arc<Dataset>, ids: Vec<LocationId>) -> Rc<Self> {
        let symbols: Vec<SharedString> = dataset
            .stored
            .dictionary
            .iter()
            .map(String::as_str)
            .map(SharedString::from)
            .collect();
        let mut labels = symbols.clone();
        for localized in &dataset.stored.localizations {
            let Ok(key) = usize::try_from(localized.key.0) else {
                continue;
            };
            let Ok(value) = usize::try_from(localized.value.0) else {
                continue;
            };
            if let (Some(target), Some(label)) = (labels.get_mut(key), symbols.get(value)) {
                *target = label.clone();
            }
        }
        Rc::new(Self {
            ids: RefCell::new(ids),
            dataset: Arc::clone(dataset),
            symbols,
            labels,
            notify: ModelNotify::default(),
        })
    }

    pub(super) fn reset(&self, ids: Vec<LocationId>) -> Result<(), AppError> {
        let mut current = self
            .ids
            .try_borrow_mut()
            .map_err(|error| AppError::Ui(format!("result model is already borrowed: {error}")))?;
        *current = ids;
        drop(current);
        self.notify.reset();
        Ok(())
    }

    pub(super) fn id_at(&self, row: usize) -> Option<LocationId> {
        self.ids
            .try_borrow()
            .ok()
            .and_then(|ids| ids.get(row).copied())
    }

    pub(super) fn position(&self, id: LocationId) -> Option<usize> {
        self.ids
            .try_borrow()
            .ok()
            .and_then(|ids| ids.iter().position(|candidate| *candidate == id))
    }
}

impl Model for ResultModel {
    type Data = LocationRow;

    fn row_count(&self) -> usize {
        self.ids.try_borrow().map_or(0, |ids| ids.len())
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        let id = self.id_at(row)?;
        let record = self.dataset.location(id)?;
        Some(display_row(&self.symbols, &self.labels, record))
    }

    fn model_tracker(&self) -> &dyn ModelTracker {
        &self.notify
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn display_row(
    symbols: &[SharedString],
    labels: &[SharedString],
    record: &LocationRecord,
) -> LocationRow {
    let [red, green, blue] = record.color.components();
    LocationRow {
        id: i32::try_from(record.id.0).unwrap_or(i32::MAX),
        color: Color::from_rgb_u8(red, green, blue),
        hex: record.color.hex().into(),
        name: shared(symbols, Some(record.name)),
        key: shared(symbols, Some(record.key)),
        kind: record.kind.label().into(),
        topography: shared(labels, Some(record.topography)),
        vegetation: shared(labels, record.vegetation),
        climate: shared(labels, record.climate),
        continent: shared(labels, Some(record.hierarchy.continent)),
        subcontinent: shared(labels, Some(record.hierarchy.subcontinent)),
        region: shared(labels, Some(record.hierarchy.region)),
        area: shared(labels, Some(record.hierarchy.area)),
        province: shared(labels, Some(record.hierarchy.province)),
        religion: shared(labels, record.religion),
        culture: shared(labels, record.culture),
        raw_material: shared(labels, record.raw_material),
        food_raw_material: symbol_matches(symbols, record.raw_material, is_food_producing),
        precious_raw_material: symbol_matches(symbols, record.raw_material, is_gold_or_silver),
        modifier: shared(labels, record.modifier),
        rgb: format!("rgb({red}, {green}, {blue})").into(),
        coastal: if record.coastal { "Yes" } else { "No" }.into(),
        river_presence: if record.river.is_some() {
            "Present"
        } else {
            "Missing"
        }
        .into(),
        river_level: record.river.map_or_else(
            || SharedString::from("-"),
            |river| format!("L{}  +{}%", river.level.0, river.level.0 * 10).into(),
        ),
        harbor: record.harbor_suitability.map_or_else(
            || SharedString::from("-"),
            |value| format!("{value:.2}").into(),
        ),
        movement_presence: if record.movement_assistance.is_some() {
            "Present"
        } else {
            "Missing"
        }
        .into(),
        movement_x: record.movement_assistance.map_or_else(
            || SharedString::from("-"),
            |value| format!("{:.2}", value[0]).into(),
        ),
        movement_y: record.movement_assistance.map_or_else(
            || SharedString::from("-"),
            |value| format!("{:.2}", value[1]).into(),
        ),
        static_capacity: record.static_population_capacity.map_or_else(
            || SharedString::from("-"),
            |value| format_population(value.total.0).into(),
        ),
        equator_capacity: record.static_population_capacity.map_or_else(
            || SharedString::from("-"),
            |value| format_population(value.equator.0).into(),
        ),
    }
}

fn shared(values: &[SharedString], symbol: Option<SymbolId>) -> SharedString {
    symbol
        .and_then(|id| usize::try_from(id.0).ok())
        .and_then(|index| values.get(index))
        .cloned()
        .unwrap_or_else(|| SharedString::from("-"))
}

fn symbol_matches(
    values: &[SharedString],
    symbol: Option<SymbolId>,
    predicate: impl FnOnce(&str) -> bool,
) -> bool {
    symbol
        .and_then(|id| usize::try_from(id.0).ok())
        .and_then(|index| values.get(index))
        .is_some_and(|value| predicate(value.as_str()))
}

pub(super) fn format_population(value: u32) -> String {
    let digits = value.to_string();
    let mut output = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, character) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            output.push(',');
        }
        output.push(character);
    }
    output
}

pub(super) fn text(dataset: &Dataset, symbol: Option<SymbolId>) -> &str {
    symbol.and_then(|id| dataset.symbol(id)).unwrap_or("-")
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use super::ResultModel;
    use crate::model::{
        Dataset, EU5_APP_ID, FORMAT_VERSION, Hierarchy, LocationId, LocationKind, LocationRecord,
        MapColor, RiverWidthMetadata, StoredDataset, SymbolId,
    };

    #[test]
    fn reset_changes_only_visible_ids() {
        let dataset = Arc::new(fixture());
        let model = ResultModel::new(&dataset, vec![LocationId(0)]);
        assert_eq!(slint::Model::row_count(model.as_ref()), 1);
        assert!(model.reset(Vec::new()).is_ok());
        assert_eq!(slint::Model::row_count(model.as_ref()), 0);
    }

    #[test]
    fn rows_classify_food_and_precious_raw_materials() {
        for (material, food, precious) in [
            ("wheat", true, false),
            ("fur", true, false),
            ("goods_gold", false, true),
            ("goods_silver", false, true),
            ("iron", false, false),
        ] {
            let mut dataset = fixture();
            dataset.stored.dictionary.push(material.to_owned());
            dataset.stored.locations[0].raw_material = Some(SymbolId(1));
            let dataset = Arc::new(dataset);
            let model = ResultModel::new(&dataset, vec![LocationId(0)]);
            assert!(
                slint::Model::row_data(model.as_ref(), 0).is_some_and(|row| {
                    row.food_raw_material == food && row.precious_raw_material == precious
                })
            );
        }
    }

    fn fixture() -> Dataset {
        let stored = StoredDataset {
            format_version: FORMAT_VERSION,
            app_id: EU5_APP_ID,
            build_id: 1,
            river_widths: RiverWidthMetadata {
                level_count: 1,
                width_min: 1.0,
                width_max: 1.0,
            },
            dictionary: vec!["a".to_owned()],
            localizations: Vec::new(),
            locations: vec![LocationRecord {
                id: LocationId(0),
                key: SymbolId(0),
                name: SymbolId(0),
                kind: LocationKind::Land,
                color: MapColor(1),
                topography: SymbolId(0),
                vegetation: None,
                climate: None,
                religion: None,
                culture: None,
                raw_material: None,
                modifier: None,
                harbor_suitability: None,
                movement_assistance: None,
                hierarchy: Hierarchy {
                    continent: SymbolId(0),
                    subcontinent: SymbolId(0),
                    region: SymbolId(0),
                    area: SymbolId(0),
                    province: SymbolId(0),
                },
                coastal: false,
                connected_sea: None,
                river: None,
                static_population_capacity: None,
            }],
            diagnostics: Vec::new(),
        };
        Dataset {
            stored,
            by_key: HashMap::new(),
            by_color: HashMap::new(),
            localized: HashMap::new(),
        }
    }
}
