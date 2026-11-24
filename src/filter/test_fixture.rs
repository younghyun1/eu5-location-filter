//! Shared synthetic records and language packs for filter and pack tests.

use std::collections::HashMap;

use crate::lang_pack::{
    GameLanguage, PACK_FORMAT_VERSION, PackLabel, PackStringId, StoredLanguagePack,
    StoredLanguageSection, dictionary_digest, resolve_text,
};
use crate::model::{
    Dataset, EU5_APP_ID, FORMAT_VERSION, Hierarchy, LocationId, LocationKind, LocationRecord,
    MapColor, RiverData, RiverLevel, RiverWidthMetadata, StoredDataset, SymbolId,
};

/// Two records: a coastal land location keyed `eire` named `Éire`, and an impassable one.
pub(crate) fn fixture() -> Dataset {
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

/// Korean names for both fixture records and a Korean label for `flatland`.
///
/// Built by hand rather than through the importer's `build_pack`, so filter and pack tests do
/// not depend on the desktop-only builder.
pub(crate) fn fixture_pack(dataset: &Dataset) -> StoredLanguagePack {
    pack_with(
        dataset,
        GameLanguage::Korean,
        &[Some("에이레"), Some("황무지")],
        &[(SymbolId(2), "평지")],
    )
}

/// Builds a valid pack: `names` per location (`None` keeps English, missing entries too) and
/// `labels` per symbol in any order. Repeated text shares one string.
pub(crate) fn pack_with(
    dataset: &Dataset,
    language: GameLanguage,
    names: &[Option<&str>],
    labels: &[(SymbolId, &str)],
) -> StoredLanguagePack {
    let mut strings: Vec<String> = Vec::new();
    let mut intern = |value: &str| {
        let slot = match strings.iter().position(|existing| existing == value) {
            Some(slot) => slot,
            None => {
                strings.push(value.to_owned());
                strings.len() - 1
            }
        };
        PackStringId(u32::try_from(slot).unwrap_or(u32::MAX))
    };
    let location_names = (0..dataset.stored.locations.len())
        .map(|slot| names.get(slot).copied().flatten().map(&mut intern))
        .collect();
    let mut sorted = labels.to_vec();
    sorted.sort_by_key(|(key, _)| *key);
    let labels = sorted
        .into_iter()
        .map(|(key, value)| PackLabel {
            key,
            value: intern(value),
        })
        .collect();
    let mut pack = StoredLanguagePack {
        format_version: PACK_FORMAT_VERSION,
        app_id: EU5_APP_ID,
        build_id: dataset.stored.build_id,
        location_count: u32::try_from(dataset.stored.locations.len()).unwrap_or(u32::MAX),
        symbol_count: u32::try_from(dataset.stored.dictionary.len()).unwrap_or(u32::MAX),
        dictionary_digest: dictionary_digest(&dataset.stored.dictionary),
        language,
        strings,
        location_names,
        labels,
        diagnostics: Vec::new(),
        section: StoredLanguageSection::default(),
    };
    pack.section = section_for(dataset, &pack);
    pack
}

/// Recomputes a pack's section from its own text, as the importer does.
pub(crate) fn section_for(dataset: &Dataset, pack: &StoredLanguagePack) -> StoredLanguageSection {
    let text = resolve_text(dataset, Some(pack));
    super::build_language_section(dataset, &text)
}

/// Gives `symbol` an English label by appending it to the dictionary, as an import does.
pub(crate) fn add_english_label(dataset: &mut Dataset, symbol: SymbolId, label: &str) {
    let value = SymbolId(u32::try_from(dataset.stored.dictionary.len()).unwrap_or(u32::MAX));
    dataset.stored.dictionary.push(label.to_owned());
    dataset
        .stored
        .localizations
        .push(crate::model::LocalizedValue { key: symbol, value });
    dataset.localized.insert(symbol, value);
}
