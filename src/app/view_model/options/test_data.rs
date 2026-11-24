//! Synthetic dataset shared by the view model tests: four locations covering missing values,
//! food, precious and plain raw materials, Latin diacritics and non-Latin labels.

use std::collections::HashMap;

use crate::app::i18n::Locale;
use crate::app::view_model::{FacetOptions, RowContext};
use crate::lang_pack::{GameLanguage, Labels, ValidatedLanguagePack, validate_pack};
use crate::model::{
    Dataset, EU5_APP_ID, FORMAT_VERSION, Hierarchy, LocalizedValue, LocationId, LocationKind,
    LocationRecord, MapColor, PopulationAmount, RiverData, RiverLevel, RiverWidthMetadata,
    StaticPopulationCapacity, StoredDataset, SymbolId,
};

/// Symbol of `hills`.
pub(in crate::app::view_model) const HILLS: SymbolId = SymbolId(2);
/// Symbol of `mountains`.
pub(in crate::app::view_model) const MOUNTAINS: SymbolId = SymbolId(3);
/// Symbol of `wheat`, a food-producing raw material.
pub(in crate::app::view_model) const WHEAT: SymbolId = SymbolId(4);
/// Symbol of `goods_gold`, a precious raw material.
pub(in crate::app::view_model) const GOLD: SymbolId = SymbolId(5);
/// Symbol of `iron`, a plain raw material.
pub(in crate::app::view_model) const IRON: SymbolId = SymbolId(6);
/// Symbol of `europe`, labelled `Europe`.
pub(in crate::app::view_model) const EUROPE: SymbolId = SymbolId(7);
/// Symbol of `forest`.
pub(in crate::app::view_model) const FOREST: SymbolId = SymbolId(9);
/// Symbol of `boreal`.
pub(in crate::app::view_model) const BOREAL: SymbolId = SymbolId(10);
/// Symbol of `catholic`.
pub(in crate::app::view_model) const CATHOLIC: SymbolId = SymbolId(11);
/// Symbol of `kourim`, labelled `Kouřim`.
pub(in crate::app::view_model) const KOURIM: SymbolId = SymbolId(12);
/// Symbol of `ras_al_ain`, labelled `Ra's al-'Ain`.
pub(in crate::app::view_model) const RAS: SymbolId = SymbolId(14);
/// Symbol of `moskva`, labelled in Cyrillic.
pub(in crate::app::view_model) const MOSKVA: SymbolId = SymbolId(16);
/// Symbol of `tokyo`, labelled in Japanese.
pub(in crate::app::view_model) const TOKYO: SymbolId = SymbolId(18);

/// Dictionary; a label follows its identifier where the two are paired in `LABELLED`.
const DICTIONARY: [&str; 27] = [
    "stockholm",
    "Stockholm",
    "hills",
    "mountains",
    "wheat",
    "goods_gold",
    "iron",
    "europe",
    "Europe",
    "forest",
    "boreal",
    "catholic",
    "kourim",
    "Kouřim",
    "ras_al_ain",
    "Ra's al-'Ain",
    "moskva",
    "Москва",
    "tokyo",
    "東京",
    "baltic_sea",
    "Baltic Sea",
    "ocean",
    "gold_mine",
    "Gold Mine",
    "iron_hill",
    "Iron Hill",
];

/// Pairs of an identifier and the dictionary index of its English label.
const LABELLED: [(u32, u32); 5] = [(7, 8), (12, 13), (14, 15), (16, 17), (18, 19)];

fn record(id: u32, key: u32, name: u32, topography: SymbolId) -> LocationRecord {
    LocationRecord {
        id: LocationId(id),
        key: SymbolId(key),
        name: SymbolId(name),
        kind: LocationKind::Land,
        color: MapColor(0x0001_0203 + id),
        topography,
        vegetation: None,
        climate: None,
        religion: None,
        culture: None,
        raw_material: None,
        modifier: None,
        harbor_suitability: None,
        movement_assistance: None,
        hierarchy: Hierarchy {
            continent: EUROPE,
            subcontinent: KOURIM,
            region: RAS,
            area: MOSKVA,
            province: TOKYO,
        },
        coastal: false,
        connected_sea: None,
        river: None,
        static_population_capacity: None,
    }
}

/// Four locations: Stockholm (every value present), the Baltic (every optional value missing),
/// a gold mine and an iron hill.
pub(in crate::app::view_model) fn dataset() -> Dataset {
    let mut stockholm = record(0, 0, 1, HILLS);
    stockholm.color = MapColor(0x1A_2B3C);
    stockholm.vegetation = Some(FOREST);
    stockholm.climate = Some(BOREAL);
    stockholm.religion = Some(CATHOLIC);
    stockholm.raw_material = Some(WHEAT);
    stockholm.harbor_suitability = Some(0.5);
    stockholm.movement_assistance = Some([-1.0, 1.25]);
    stockholm.coastal = true;
    stockholm.connected_sea = Some(LocationId(1));
    stockholm.river = Some(RiverData {
        level: RiverLevel(2),
        has_source: true,
        has_confluence: false,
    });
    stockholm.static_population_capacity = Some(StaticPopulationCapacity {
        vegetation: PopulationAmount(1_200),
        equator: PopulationAmount(34_000),
        modifier_basis_points: -25,
        total: PopulationAmount(1_234_567),
    });
    let mut baltic = record(1, 20, 21, SymbolId(22));
    baltic.kind = LocationKind::Sea;
    baltic.color = MapColor(0x00_00FF);
    let mut mine = record(2, 23, 24, MOUNTAINS);
    mine.raw_material = Some(GOLD);
    mine.river = Some(RiverData {
        level: RiverLevel(5),
        has_source: false,
        has_confluence: true,
    });
    mine.static_population_capacity = Some(StaticPopulationCapacity {
        vegetation: PopulationAmount(0),
        equator: PopulationAmount(0),
        modifier_basis_points: 150,
        total: PopulationAmount(0),
    });
    let mut hill = record(3, 25, 26, HILLS);
    hill.raw_material = Some(IRON);
    hill.river = Some(RiverData {
        level: RiverLevel(9),
        has_source: false,
        has_confluence: false,
    });
    let locations = vec![stockholm, baltic, mine, hill];
    let by_key = locations.iter().map(|record| (record.key, record.id)).collect();
    let by_color = locations.iter().map(|record| (record.color, record.id)).collect();
    let localizations: Vec<LocalizedValue> = LABELLED
        .iter()
        .map(|(key, value)| LocalizedValue {
            key: SymbolId(*key),
            value: SymbolId(*value),
        })
        .collect();
    let localized: HashMap<SymbolId, SymbolId> = localizations
        .iter()
        .map(|entry| (entry.key, entry.value))
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
            dictionary: DICTIONARY.iter().map(|text| (*text).to_owned()).collect(),
            localizations,
            locations,
            diagnostics: Vec::new(),
        },
        by_key,
        by_color,
        localized,
    }
}

/// English option lists of `dataset`.
pub(in crate::app::view_model) fn english_options(dataset: &Dataset) -> FacetOptions {
    FacetOptions::build(Labels::new(dataset, None), Locale::EnUs)
}

/// English cell context of `dataset`.
pub(in crate::app::view_model) fn english_context(dataset: &Dataset) -> RowContext<'_> {
    RowContext {
        labels: Labels::new(dataset, None),
        locale: Locale::EnUs,
    }
}

/// A validated Korean pack naming the first two locations and labelling `hills` and `wheat`;
/// `None` when validation rejects the synthetic pack.
pub(in crate::app::view_model) fn korean_pack(dataset: &Dataset) -> Option<ValidatedLanguagePack> {
    let pack = crate::filter::test_fixture::pack_with(
        dataset,
        GameLanguage::Korean,
        &[Some("스톡홀름"), Some("발트해")],
        &[(HILLS, "언덕"), (WHEAT, "밀")],
    );
    validate_pack(pack, dataset, GameLanguage::Korean).ok()
}
