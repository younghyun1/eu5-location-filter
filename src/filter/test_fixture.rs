//! Shared synthetic records for filter tests.

use std::collections::HashMap;

use crate::model::{
    Dataset, EU5_APP_ID, FORMAT_VERSION, Hierarchy, LocationId, LocationKind, LocationRecord,
    MapColor, RiverData, RiverLevel, RiverWidthMetadata, StoredDataset, SymbolId,
};

pub(super) fn fixture() -> Dataset {
    let locations = vec![
        record(
            0,
            LocationKind::Land,
            true,
            Some(0.5),
            None,
            Some(RiverLevel(1)),
        ),
        record(
            1,
            LocationKind::Impassable,
            false,
            None,
            Some(SymbolId(3)),
            None,
        ),
    ];
    let stored = StoredDataset {
        format_version: FORMAT_VERSION,
        app_id: EU5_APP_ID,
        build_id: 1,
        river_widths: RiverWidthMetadata {
            level_count: 2,
            width_min: 1.0,
            width_max: 2.0,
        },
        dictionary: vec![
            "eire".to_owned(),
            "Éire".to_owned(),
            "flatland".to_owned(),
            "catholic".to_owned(),
            "waste".to_owned(),
        ],
        localizations: Vec::new(),
        locations,
        diagnostics: Vec::new(),
    };
    Dataset {
        stored,
        by_key: HashMap::from([(SymbolId(0), LocationId(0)), (SymbolId(4), LocationId(1))]),
        by_color: HashMap::from([(MapColor(1), LocationId(0)), (MapColor(2), LocationId(1))]),
        localized: HashMap::new(),
    }
}

fn record(
    id: u32,
    kind: LocationKind,
    coastal: bool,
    harbor: Option<f32>,
    religion: Option<SymbolId>,
    river: Option<RiverLevel>,
) -> LocationRecord {
    LocationRecord {
        id: LocationId(id),
        key: if id == 0 { SymbolId(0) } else { SymbolId(4) },
        name: if id == 0 { SymbolId(1) } else { SymbolId(4) },
        kind,
        color: MapColor(id + 1),
        topography: SymbolId(2),
        vegetation: None,
        climate: None,
        religion,
        culture: None,
        raw_material: None,
        modifier: None,
        harbor_suitability: harbor,
        movement_assistance: None,
        hierarchy: Hierarchy {
            continent: SymbolId(0),
            subcontinent: SymbolId(0),
            region: SymbolId(0),
            area: SymbolId(0),
            province: SymbolId(0),
        },
        coastal,
        connected_sea: None,
        river: river.map(|level| RiverData {
            level,
            has_source: false,
            has_confluence: false,
        }),
        static_population_capacity: None,
    }
}
