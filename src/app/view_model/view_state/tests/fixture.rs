//! Synthetic dataset and helpers shared by the view-state tests.

use std::collections::HashMap;

use crate::app::i18n::Locale;
use crate::app::view_model::view_state::{
    LocationKeys, ResolvedView, ViewSnapshot, decode, encode_into, resolve,
};
use crate::app::view_model::{FacetOptions, SortState};
use crate::filter::FilterSet;
use crate::lang_pack::Labels;
use crate::model::{
    Dataset, EU5_APP_ID, FORMAT_VERSION, Hierarchy, LocationId, LocationKind, LocationRecord,
    MapColor, RiverData, RiverLevel, RiverWidthMetadata, StoredDataset, SymbolId,
};

/// Dictionary symbols of the fixture.
pub(super) const STOCKHOLM: SymbolId = SymbolId(0);
pub(super) const HILLS: SymbolId = SymbolId(2);
pub(super) const MOUNTAINS: SymbolId = SymbolId(3);
pub(super) const WHEAT: SymbolId = SymbolId(4);
pub(super) const UPPLAND: SymbolId = SymbolId(5);
pub(super) const EUROPE: SymbolId = SymbolId(6);
pub(super) const CATHOLIC: SymbolId = SymbolId(7);
pub(super) const BOREAL: SymbolId = SymbolId(8);
pub(super) const FOREST: SymbolId = SymbolId(9);
pub(super) const EVORA: SymbolId = SymbolId(13);
/// First of `LONG_KEY_COUNT` unused symbols with 60-byte keys, for the length cap.
pub(super) const FIRST_LONG_KEY: u32 = 15;
/// Number of long keys.
pub(super) const LONG_KEY_COUNT: u32 = 40;

/// Three locations: Stockholm (land, hills), the Baltic (sea) and Évora (land, mountains).
pub(super) fn dataset() -> Dataset {
    let mut dictionary: Vec<String> = [
        "stockholm",
        "Stockholm",
        "hills",
        "mountains",
        "wheat",
        "uppland",
        "europe",
        "catholic",
        "boreal",
        "forest",
        "ocean",
        "baltic_sea",
        "Baltic Sea",
        "évora",
        "Évora",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    dictionary
        .extend((0..LONG_KEY_COUNT).map(|index| format!("long_key_{index:02}_{}", "x".repeat(47))));
    let hierarchy = Hierarchy {
        continent: EUROPE,
        subcontinent: UPPLAND,
        region: UPPLAND,
        area: UPPLAND,
        province: UPPLAND,
    };
    let stockholm = LocationRecord {
        id: LocationId(0),
        key: STOCKHOLM,
        name: SymbolId(1),
        kind: LocationKind::Land,
        color: MapColor(0x1A_2B3C),
        topography: HILLS,
        vegetation: Some(FOREST),
        climate: Some(BOREAL),
        religion: Some(CATHOLIC),
        culture: None,
        raw_material: Some(WHEAT),
        modifier: None,
        harbor_suitability: Some(0.5),
        movement_assistance: Some([-1.0, 1.0]),
        hierarchy,
        coastal: true,
        connected_sea: Some(LocationId(1)),
        river: Some(RiverData {
            level: RiverLevel(2),
            has_source: false,
            has_confluence: false,
        }),
        static_population_capacity: None,
    };
    let baltic = LocationRecord {
        id: LocationId(1),
        key: SymbolId(11),
        name: SymbolId(12),
        kind: LocationKind::Sea,
        color: MapColor(0x00_00FF),
        topography: SymbolId(10),
        vegetation: None,
        climate: None,
        religion: None,
        raw_material: None,
        harbor_suitability: None,
        movement_assistance: None,
        coastal: false,
        connected_sea: None,
        river: None,
        ..stockholm.clone()
    };
    let evora = LocationRecord {
        id: LocationId(2),
        key: EVORA,
        name: SymbolId(14),
        color: MapColor(0x33_4455),
        topography: MOUNTAINS,
        climate: None,
        raw_material: None,
        river: Some(RiverData {
            level: RiverLevel(3),
            has_source: true,
            has_confluence: false,
        }),
        ..stockholm.clone()
    };
    let locations = vec![stockholm, baltic, evora];
    let by_key = locations
        .iter()
        .map(|record| (record.key, record.id))
        .collect();
    let by_color = locations
        .iter()
        .map(|record| (record.color, record.id))
        .collect();
    Dataset {
        stored: StoredDataset {
            format_version: FORMAT_VERSION,
            app_id: EU5_APP_ID,
            build_id: 1,
            river_widths: RiverWidthMetadata {
                level_count: 5,
                width_min: 1.0,
                width_max: 2.0,
            },
            dictionary,
            localizations: Vec::new(),
            locations,
            diagnostics: Vec::new(),
        },
        by_key,
        by_color,
        localized: HashMap::new(),
    }
}

/// Everything resolution needs, built once per test.
pub(super) struct Loaded {
    /// The fixture dataset.
    pub(super) dataset: Dataset,
    /// Option lists in English.
    pub(super) options: FacetOptions,
    /// Sorted location keys.
    pub(super) keys: LocationKeys,
}

impl Loaded {
    /// Builds options and keys for the fixture.
    pub(super) fn new() -> Self {
        let dataset = dataset();
        let options = FacetOptions::build(Labels::new(&dataset, None), Locale::EnUs);
        let keys = LocationKeys::build(&dataset);
        Self {
            dataset,
            options,
            keys,
        }
    }

    /// Decodes and resolves a string; `None` when the whole string is rejected.
    pub(super) fn resolve(&self, text: &str) -> Option<ResolvedView> {
        let raw = decode(text).ok()?;
        Some(resolve(&raw, &self.options, &self.keys, &self.dataset))
    }

    /// Canonical string of a resolved view.
    pub(super) fn encode_view(&self, view: &ResolvedView) -> String {
        let key = view
            .selected
            .and_then(|id| self.dataset.location(id))
            .and_then(|record| self.dataset.symbol(record.key));
        encode(&self.dataset, &view.filters, view.sort, key)
    }

    /// Decode, resolve and encode again; `None` when the whole string is rejected.
    pub(super) fn canonical(&self, text: &str) -> Option<String> {
        self.resolve(text).map(|view| self.encode_view(&view))
    }
}

/// Canonical string of a state.
pub(super) fn encode(
    dataset: &Dataset,
    filters: &FilterSet,
    sort: SortState,
    selected_key: Option<&str>,
) -> String {
    let mut out = String::from("stale content");
    encode_into(
        &mut out,
        &ViewSnapshot {
            filters,
            sort,
            selected_key,
            dataset,
        },
    );
    out
}
