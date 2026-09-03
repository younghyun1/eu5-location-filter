use std::collections::{HashMap, HashSet};

use slint::Model;

use super::{filter_source, model, static_model};
use crate::model::{
    Dataset, EU5_APP_ID, FORMAT_VERSION, Hierarchy, LocalizedValue, LocationId, LocationKind,
    LocationRecord, MapColor, RiverWidthMetadata, StoredDataset, SymbolId,
};

#[test]
fn options_are_sorted_and_include_internal_ids() {
    let dataset = fixture();
    let result = model(&dataset, HashSet::from([SymbolId(0)]), true);
    assert_eq!(result.row_data(0).as_deref(), Some("Any"));
    assert_eq!(result.row_data(1).as_deref(), Some("Missing"));
    assert_eq!(result.row_data(2).as_deref(), Some("Flatland  [flatland]"));
}

#[test]
fn option_search_is_ascii_folded() {
    let options = static_model(&["Kouřim  [kourim]", "Stockholm  [stockholm]"]);
    let result = filter_source(options, "kourim");
    assert_eq!(result.row_count(), 1);
    assert_eq!(result.row_data(0).as_deref(), Some("Kouřim  [kourim]"));
}

fn fixture() -> Dataset {
    let record = LocationRecord {
        id: LocationId(0),
        key: SymbolId(0),
        name: SymbolId(1),
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
    };
    Dataset {
        stored: StoredDataset {
            format_version: FORMAT_VERSION,
            app_id: EU5_APP_ID,
            build_id: 1,
            river_widths: RiverWidthMetadata {
                level_count: 1,
                width_min: 1.0,
                width_max: 1.0,
            },
            dictionary: vec!["flatland".to_owned(), "Flatland".to_owned()],
            localizations: vec![LocalizedValue {
                key: SymbolId(0),
                value: SymbolId(1),
            }],
            locations: vec![record],
            diagnostics: Vec::new(),
        },
        by_key: HashMap::new(),
        by_color: HashMap::new(),
        localized: HashMap::from([(SymbolId(0), SymbolId(1))]),
    }
}
