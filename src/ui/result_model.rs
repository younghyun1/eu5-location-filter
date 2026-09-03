//! Virtualized Slint model backed by filtered location IDs and prebuilt rows.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use slint::{Color, Model, ModelNotify, ModelTracker, SharedString};

use super::LocationRow;
use crate::AppError;
use crate::model::{Dataset, LocationId, LocationRecord, SymbolId};

/// The only mutable result state is the compact vector of visible IDs.
pub(super) struct ResultModel {
    ids: RefCell<Vec<LocationId>>,
    rows: Vec<LocationRow>,
    notify: ModelNotify,
}

impl ResultModel {
    pub(super) fn new(dataset: &Arc<Dataset>, ids: Vec<LocationId>) -> Rc<Self> {
        let rows = dataset
            .stored
            .locations
            .iter()
            .map(|record| display_row(dataset, record))
            .collect();
        Rc::new(Self {
            ids: RefCell::new(ids),
            rows,
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
        usize::try_from(id.0)
            .ok()
            .and_then(|index| self.rows.get(index))
            .cloned()
    }

    fn model_tracker(&self) -> &dyn ModelTracker {
        &self.notify
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn display_row(dataset: &Dataset, record: &LocationRecord) -> LocationRow {
    let [red, green, blue] = record.color.components();
    LocationRow {
        id: i32::try_from(record.id.0).unwrap_or(i32::MAX),
        color: Color::from_rgb_u8(red, green, blue),
        hex: record.color.hex().into(),
        name: text(dataset, Some(record.name)).into(),
        key: text(dataset, Some(record.key)).into(),
        kind: record.kind.label().into(),
        topography: text(dataset, Some(record.topography)).into(),
        vegetation: text(dataset, record.vegetation).into(),
        climate: text(dataset, record.climate).into(),
        continent: label(dataset, Some(record.hierarchy.continent)).into(),
        subcontinent: label(dataset, Some(record.hierarchy.subcontinent)).into(),
        region: label(dataset, Some(record.hierarchy.region)).into(),
        area: label(dataset, Some(record.hierarchy.area)).into(),
        province: label(dataset, Some(record.hierarchy.province)).into(),
        religion: label(dataset, record.religion).into(),
        culture: label(dataset, record.culture).into(),
        raw_material: label(dataset, record.raw_material).into(),
        modifier: label(dataset, record.modifier).into(),
        rgb: format!("rgb({red}, {green}, {blue})").into(),
        coastal: if record.coastal { "Yes" } else { "No" }.into(),
        river_presence: if record.river.is_some() {
            "Present"
        } else {
            "Missing"
        }
        .into(),
        river_level: record.river.map_or_else(
            || SharedString::from("—"),
            |river| river.level.0.to_string().into(),
        ),
        harbor: record.harbor_suitability.map_or_else(
            || SharedString::from("—"),
            |value| format!("{value:.2}").into(),
        ),
        movement_presence: if record.movement_assistance.is_some() {
            "Present"
        } else {
            "Missing"
        }
        .into(),
        movement_x: record.movement_assistance.map_or_else(
            || SharedString::from("—"),
            |value| format!("{:.2}", value[0]).into(),
        ),
        movement_y: record.movement_assistance.map_or_else(
            || SharedString::from("—"),
            |value| format!("{:.2}", value[1]).into(),
        ),
    }
}

pub(super) fn text(dataset: &Dataset, symbol: Option<SymbolId>) -> &str {
    symbol.and_then(|id| dataset.symbol(id)).unwrap_or("—")
}

fn label(dataset: &Dataset, symbol: Option<SymbolId>) -> &str {
    symbol.and_then(|id| dataset.label(id)).unwrap_or("—")
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
