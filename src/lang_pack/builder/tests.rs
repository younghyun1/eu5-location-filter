use std::collections::HashMap;
use std::fs;

use super::strings::has_unresolved_reference;
use super::{build_pack, import_pack, rebuild_pack, requested_keys};
use crate::AppError;
use crate::filter::test_fixture::fixture;
use crate::lang_pack::{GameLanguage, PackStringId, encode_pack, validate_pack};
use crate::model::{Dataset, SymbolId};
use crate::steam::GameInstallation;

fn values(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect()
}

fn korean() -> HashMap<String, String> {
    values(&[
        ("eire", "에이레"),
        ("waste", "황무지"),
        ("flatland", "평지"),
        ("catholic", "가톨릭"),
    ])
}

/// Gives the first fixture record a static modifier `coastal_mod` as symbol 5.
fn with_modifier() -> Dataset {
    let mut dataset = fixture();
    dataset.stored.dictionary.push("coastal_mod".to_owned());
    if let Some(record) = dataset.stored.locations.first_mut() {
        record.modifier = Some(SymbolId(5));
    }
    dataset
}

#[test]
fn requested_keys_cover_names_labels_and_modifier_prefixes() -> Result<(), AppError> {
    let keys = requested_keys(&with_modifier())?;
    for key in ["eire", "waste", "flatland", "catholic"] {
        assert!(keys.contains(key), "{key}");
    }
    assert!(keys.contains("STATIC_MODIFIER_NAME_coastal_mod"));
    assert!(!keys.contains("coastal_mod"));
    assert_eq!(keys.len(), 5);
    Ok(())
}

#[test]
fn builds_a_valid_pack_with_labels_for_location_key_symbols() -> Result<(), AppError> {
    let dataset = fixture();
    let pack = build_pack(&dataset, GameLanguage::Korean, &korean())?;
    // The hierarchy symbol `eire` is also a location key: it keeps its label, sharing the
    // string with the location name.
    let keys: Vec<SymbolId> = pack.labels.iter().map(|label| label.key).collect();
    assert_eq!(keys, vec![SymbolId(0), SymbolId(2), SymbolId(3)]);
    assert_eq!(pack.labels.first().map(|label| label.value), Some(PackStringId(0)));
    assert_eq!(pack.location_names, vec![Some(PackStringId(0)), Some(PackStringId(1))]);
    assert_eq!(pack.strings, vec!["에이레", "황무지", "평지", "가톨릭"]);
    assert!(pack.diagnostics.is_empty());
    let validated = validate_pack(pack, &dataset, GameLanguage::Korean)?;
    assert_eq!(validated.label(SymbolId(2)), Some("평지"));
    Ok(())
}

#[test]
fn text_equal_to_english_and_missing_text_stay_out_of_the_pack() -> Result<(), AppError> {
    let dataset = fixture();
    let french = values(&[("eire", "Éire"), ("waste", "waste"), ("flatland", "flatland")]);
    let pack = build_pack(&dataset, GameLanguage::French, &french)?;
    assert_eq!(pack.location_names, vec![None, None]);
    // `eire` as a label differs: its English label is the identifier, not the name.
    assert_eq!(pack.strings, vec!["Éire"]);
    assert_eq!(pack.labels.len(), 1);
    let codes: Vec<(&str, u32)> = pack
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.code.as_str(), diagnostic.count))
        .collect();
    assert_eq!(codes, vec![("missing_label", 1)]);
    assert!(validate_pack(pack, &dataset, GameLanguage::French).is_ok());
    Ok(())
}

#[test]
fn modifiers_use_their_static_modifier_key() -> Result<(), AppError> {
    let dataset = with_modifier();
    let mut text = korean();
    text.insert("coastal_mod".to_owned(), "잘못된 키".to_owned());
    text.insert("STATIC_MODIFIER_NAME_coastal_mod".to_owned(), "해안".to_owned());
    let pack = build_pack(&dataset, GameLanguage::Korean, &text)?;
    let validated = validate_pack(pack, &dataset, GameLanguage::Korean)?;
    assert_eq!(validated.label(SymbolId(5)), Some("해안"));
    Ok(())
}

#[test]
fn a_symbol_used_as_modifier_and_religion_is_rejected() {
    let mut dataset = fixture();
    if let Some(record) = dataset.stored.locations.first_mut() {
        record.modifier = Some(SymbolId(3));
    }
    assert!(requested_keys(&dataset).is_err());
    assert!(build_pack(&dataset, GameLanguage::Korean, &korean()).is_err());
}

#[test]
fn counts_diagnostics_and_falls_back_on_unusable_values() -> Result<(), AppError> {
    let dataset = fixture();
    let russian = values(&[
        ("eire", "$eire_short$ земля"),
        ("waste", "bad\0value"),
        ("flatland", " "),
        ("catholic", "Шведск"),
    ]);
    let mut culture = dataset.clone();
    if let Some(record) = culture.stored.locations.get_mut(1) {
        record.culture = Some(SymbolId(3));
        record.religion = None;
    }
    let pack = build_pack(&culture, GameLanguage::Russian, &russian)?;
    let codes: Vec<(&str, u32)> = pack
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.code.as_str(), diagnostic.count))
        .collect();
    assert_eq!(
        codes,
        vec![
            ("missing_location_name", 1),
            ("missing_label", 1),
            ("unresolved_reference", 2),
            ("rejected_value", 1),
            ("culture_label_stem", 1),
        ]
    );
    assert_eq!(pack.location_names.get(1), Some(&None));
    assert!(validate_pack(pack, &culture, GameLanguage::Russian).is_ok());
    assert!(has_unresolved_reference("a $ b $key$"));
    assert!(!has_unresolved_reference("costs 5$ or $"));
    Ok(())
}

#[test]
fn builds_are_deterministic_and_rebuild_is_identity() -> Result<(), AppError> {
    let dataset = fixture();
    let first = build_pack(&dataset, GameLanguage::Korean, &korean())?;
    let second = build_pack(&dataset, GameLanguage::Korean, &korean())?;
    assert_eq!(encode_pack(&first)?, encode_pack(&second)?);
    let mut stale = first.clone();
    stale.section.orders.clear();
    assert_eq!(rebuild_pack(&dataset, stale)?, first);
    let mut other = first;
    other.build_id += 1;
    assert!(matches!(
        rebuild_pack(&dataset, other),
        Err(AppError::PackMismatch(_))
    ));
    assert!(build_pack(&dataset, GameLanguage::English, &korean()).is_err());
    Ok(())
}

#[test]
fn imports_one_language_from_its_own_directory() -> Result<(), AppError> {
    let root = std::env::temp_dir().join(format!(
        "eu5-location-filter-pack-import-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let base = root.join("game/main_menu/localization");
    let files = [
        (
            "korean/location_names/location_names_l_korean.yml",
            "\u{feff}l_korean:\n eire: \"에이레\"\nwaste: \"황무지\"\n",
        ),
        (
            "korean/terrains_l_korean.yml",
            "\u{feff}l_korean:\n flatland:0 \"$plain$\"\n plain: \"평지\"\n",
        ),
        (
            "english/terrains_l_english.yml",
            "\u{feff}l_english:\n flatland: \"Wrong language\"\n",
        ),
    ];
    for (relative, text) in files {
        let path = base.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| AppError::io("create", parent, source))?;
        }
        fs::write(&path, text).map_err(|source| AppError::io("write", &path, source))?;
    }
    let installation = GameInstallation {
        root: root.clone(),
        build_id: 1,
    };
    let dataset = fixture();
    let pack = import_pack(&installation, &dataset, GameLanguage::Korean)?;
    let validated = validate_pack(pack, &dataset, GameLanguage::Korean)?;
    assert_eq!(validated.label(SymbolId(2)), Some("평지"));
    assert_eq!(validated.name(crate::model::LocationId(1)), Some("황무지"));
    assert!(matches!(
        import_pack(&installation, &dataset, GameLanguage::Japanese),
        Err(AppError::MissingFile(_))
    ));
    let _ = fs::remove_dir_all(root);
    Ok(())
}
