use std::collections::HashMap;
use std::sync::Arc;

use crate::model::{
    Dataset, EU5_APP_ID, FORMAT_VERSION, Hierarchy, LocationId, LocationKind, LocationRecord,
    MapColor, RiverData, RiverLevel, RiverWidthMetadata, StoredDataset, SymbolId,
};

use super::{FilterEngine, FilterSet, FloatRange, OptionalFacet, OptionalNumeric, SortField};

#[test]
fn filters_use_or_within_fields_and_and_across_fields() {
    let dataset = fixture();
    let engine = FilterEngine::new(Arc::new(dataset));
    let mut filters = FilterSet::default();
    filters
        .kinds
        .extend([LocationKind::Land, LocationKind::Lake]);
    filters.coastal = Some(true);
    let ids = engine.apply(&filters, SortField::Name, true);
    assert_eq!(ids, vec![LocationId(0)]);
}

#[test]
fn supports_missing_inclusive_ranges_and_unicode_search() {
    let dataset = fixture();
    let engine = FilterEngine::new(Arc::new(dataset));
    let mut filters = FilterSet {
        religion: OptionalFacet::Missing,
        harbor_presence: OptionalNumeric::Present,
        harbor_range: FloatRange {
            min: Some(0.5),
            max: Some(0.5),
        },
        search: "ÉIRE".to_owned(),
        ..FilterSet::default()
    };
    assert_eq!(
        engine.apply(&filters, SortField::Name, true),
        vec![LocationId(0)]
    );
    filters.harbor_presence = OptionalNumeric::Missing;
    assert!(engine.apply(&filters, SortField::Name, true).is_empty());
}

#[test]
fn toggles_impassables_and_preserves_visible_selection() {
    let dataset = fixture();
    let engine = FilterEngine::new(Arc::new(dataset));
    let filters = FilterSet {
        show_impassable: false,
        ..FilterSet::default()
    };
    let ids = engine.apply(&filters, SortField::RiverLevel, false);
    assert_eq!(ids, vec![LocationId(0)]);
    assert_eq!(
        FilterEngine::preserve_selection(Some(LocationId(0)), &ids),
        Some(LocationId(0))
    );
    assert_eq!(
        FilterEngine::preserve_selection(Some(LocationId(1)), &ids),
        None
    );
}

#[test]
fn numeric_bounds_exclude_missing_values() {
    let engine = FilterEngine::new(Arc::new(fixture()));
    let filters = FilterSet {
        river_level_min: Some(0),
        ..FilterSet::default()
    };
    assert_eq!(
        engine.apply(&filters, SortField::Name, true),
        vec![LocationId(0)]
    );
}

fn fixture() -> Dataset {
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
            rendered_width: f32::from(level.0),
        }),
    }
}
