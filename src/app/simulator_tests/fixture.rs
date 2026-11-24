//! A small synthetic dataset for the simulator tests.
//!
//! Names deliberately differ from identifiers and sort differently under them, so a sort by name
//! and a sort by identifier give different orders. Six locations are land, two are sea and one
//! is impassable, so the default land-only view shows six rows and selecting the sea kind adds
//! two. The search term `vik` matches exactly one name.

use std::collections::HashMap;

use crate::AppError;
use crate::app::load::{self, LoadedData};
use crate::model::{
    Dataset, EU5_APP_ID, FORMAT_VERSION, Hierarchy, LocationId, LocationRecord, MapColor,
    RiverData, RiverLevel, RiverWidthMetadata, StoredDataset, SymbolId,
};

mod rows;

use rows::{ROWS, Row};

/// Number of land locations, the size of the default result set.
pub(super) const LAND_COUNT: usize = 6;
/// Number of sea locations, added to the results when the sea kind is selected.
pub(super) const SEA_COUNT: usize = 2;
/// Number of land locations on the `europe` continent.
pub(super) const EUROPE_LAND_COUNT: usize = 3;
/// Dataset build identifier shown in the header.
pub(super) const BUILD_ID: u64 = 4_242;

/// Land locations in ascending name order.
pub(super) const LAND_BY_NAME: [LocationId; LAND_COUNT] = [
    LocationId(1),
    LocationId(3),
    LocationId(0),
    LocationId(5),
    LocationId(4),
    LocationId(2),
];

/// Land locations in ascending identifier order.
pub(super) const LAND_BY_IDENTIFIER: [LocationId; LAND_COUNT] = [
    LocationId(5),
    LocationId(4),
    LocationId(3),
    LocationId(2),
    LocationId(0),
    LocationId(1),
];

/// Dictionary that interns each distinct text once.
#[derive(Default)]
struct Dictionary {
    words: Vec<String>,
}

impl Dictionary {
    /// Symbol of `text`, appended on first use.
    fn symbol(&mut self, text: &str) -> SymbolId {
        let slot = match self.words.iter().position(|word| word == text) {
            Some(slot) => slot,
            None => {
                self.words.push(text.to_owned());
                self.words.len() - 1
            }
        };
        SymbolId(u32::try_from(slot).unwrap_or(u32::MAX))
    }
}

/// Record for identifier `index` of `ROWS`.
fn record(index: usize, row: &Row, dictionary: &mut Dictionary) -> LocationRecord {
    let id = u32::try_from(index).unwrap_or(u32::MAX);
    LocationRecord {
        id: LocationId(id),
        key: dictionary.symbol(row.key),
        name: dictionary.symbol(row.name),
        kind: row.kind,
        color: MapColor(0x10_20_30 + id * 0x01_01_01),
        topography: dictionary.symbol(row.topography),
        vegetation: None,
        climate: None,
        religion: None,
        culture: None,
        raw_material: None,
        modifier: None,
        harbor_suitability: row.harbor,
        movement_assistance: None,
        hierarchy: Hierarchy {
            continent: dictionary.symbol(row.continent),
            subcontinent: dictionary.symbol("north_reach"),
            region: dictionary.symbol("core_region"),
            area: dictionary.symbol("core_area"),
            province: dictionary.symbol("core_province"),
        },
        coastal: row.coastal,
        connected_sea: None,
        river: row.river.map(|level| RiverData {
            level: RiverLevel(level),
            has_source: false,
            has_confluence: false,
        }),
        static_population_capacity: None,
    }
}

/// The synthetic dataset, built the way the storage layer builds one from a decoded blob.
#[must_use]
pub(super) fn dataset() -> Dataset {
    let mut dictionary = Dictionary::default();
    let locations: Vec<LocationRecord> = ROWS
        .iter()
        .enumerate()
        .map(|(index, row)| record(index, row, &mut dictionary))
        .collect();
    let by_key: HashMap<SymbolId, LocationId> = locations
        .iter()
        .map(|record| (record.key, record.id))
        .collect();
    let by_color: HashMap<MapColor, LocationId> = locations
        .iter()
        .map(|record| (record.color, record.id))
        .collect();
    let stored = StoredDataset {
        format_version: FORMAT_VERSION,
        app_id: EU5_APP_ID,
        build_id: BUILD_ID,
        river_widths: RiverWidthMetadata {
            level_count: 2,
            width_min: 1.0,
            width_max: 2.0,
        },
        dictionary: dictionary.words,
        localizations: Vec::new(),
        locations,
        diagnostics: Vec::new(),
    };
    Dataset {
        stored,
        by_key,
        by_color,
        localized: HashMap::new(),
    }
}

/// The dataset with a freshly built engine, ready for `App::with_data`.
pub(super) fn loaded() -> Result<LoadedData, AppError> {
    load::prepare(dataset(), None)
}

#[cfg(test)]
mod tests {
    use super::{EUROPE_LAND_COUNT, LAND_COUNT, ROWS, SEA_COUNT, dataset};
    use crate::model::LocationKind;

    #[test]
    fn the_counts_match_the_rows() {
        let count = |kind: LocationKind| ROWS.iter().filter(|row| row.kind == kind).count();
        assert_eq!(count(LocationKind::Land), LAND_COUNT);
        assert_eq!(count(LocationKind::Sea), SEA_COUNT);
        let europe = ROWS
            .iter()
            .filter(|row| row.kind == LocationKind::Land && row.continent == "europe")
            .count();
        assert_eq!(europe, EUROPE_LAND_COUNT);
    }

    #[test]
    fn keys_and_colors_are_unique() {
        let data = dataset();
        assert_eq!(data.by_key.len(), ROWS.len());
        assert_eq!(data.by_color.len(), ROWS.len());
    }
}
