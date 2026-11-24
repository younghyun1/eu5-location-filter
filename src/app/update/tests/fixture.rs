//! A three-location dataset and the ready application built from it.

use std::collections::HashMap;

use crate::AppError;
use crate::app::context::Platform;
use crate::app::i18n::Locale;
use crate::app::load::{LoadedData, prepare};
use crate::app::state::App;
use crate::app::theme::PALETTE_DARK;
use crate::model::{
    Dataset, EU5_APP_ID, FORMAT_VERSION, Hierarchy, LocationId, LocationKind, LocationRecord,
    MapColor, RiverWidthMetadata, StoredDataset, SymbolId,
};

/// Land location `stockholm`.
pub(super) const LAND: LocationId = LocationId(0);
/// Sea location `north_sea`.
pub(super) const SEA: LocationId = LocationId(1);
/// Impassable location `alps`.
pub(super) const PEAKS: LocationId = LocationId(2);

/// Dictionary: keys and names of the three locations, then terrain and hierarchy symbols.
const DICTIONARY: [&str; 11] = [
    "stockholm",
    "Stockholm",
    "north_sea",
    "North Sea",
    "alps",
    "Alps",
    "plains",
    "ocean",
    "mountains",
    "europe",
    "uppland",
];

/// One record; every hierarchy level uses `europe` or `uppland`.
fn record(id: u32, kind: LocationKind, topography: u32) -> LocationRecord {
    LocationRecord {
        id: LocationId(id),
        key: SymbolId(id * 2),
        name: SymbolId(id * 2 + 1),
        kind,
        color: MapColor(0x10_00_00 * (id + 1)),
        topography: SymbolId(topography),
        vegetation: None,
        climate: None,
        religion: None,
        culture: None,
        raw_material: None,
        modifier: None,
        harbor_suitability: if id == 0 { Some(0.5) } else { None },
        movement_assistance: None,
        hierarchy: Hierarchy {
            continent: SymbolId(9),
            subcontinent: SymbolId(9),
            region: SymbolId(9),
            area: SymbolId(10),
            province: SymbolId(10),
        },
        coastal: id == 0,
        connected_sea: if id == 0 { Some(SEA) } else { None },
        river: None,
        static_population_capacity: None,
    }
}

/// Synthetic dataset with one land, one sea and one impassable location.
pub(super) fn dataset() -> Dataset {
    let locations = vec![
        record(0, LocationKind::Land, 6),
        record(1, LocationKind::Sea, 7),
        record(2, LocationKind::Impassable, 8),
    ];
    let by_key = locations
        .iter()
        .map(|location| (location.key, location.id))
        .collect::<HashMap<_, _>>();
    let by_color = locations
        .iter()
        .map(|location| (location.color, location.id))
        .collect::<HashMap<_, _>>();
    Dataset {
        stored: StoredDataset {
            format_version: FORMAT_VERSION,
            app_id: EU5_APP_ID,
            build_id: 7,
            river_widths: RiverWidthMetadata {
                level_count: 2,
                width_min: 1.0,
                width_max: 2.0,
            },
            dictionary: DICTIONARY.iter().map(|value| (*value).to_owned()).collect(),
            localizations: Vec::new(),
            locations,
            diagnostics: Vec::new(),
        },
        by_key,
        by_color,
        localized: HashMap::new(),
    }
}

/// Prepared dataset and engine.
pub(super) fn loaded() -> Result<LoadedData, AppError> {
    prepare(dataset(), None)
}

/// Ready application in the wide layout with English text.
pub(super) fn app(platform: Platform) -> Result<App, AppError> {
    Ok(App::with_data(platform, Locale::EnUs, PALETTE_DARK, loaded()?))
}
