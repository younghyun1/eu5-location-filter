//! Result-model behavior tests.

use std::collections::HashMap;
use std::sync::Arc;

use super::result_model::ResultModel;
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
        ("silver", false, true),
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
