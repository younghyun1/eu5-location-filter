use std::fs;

use crate::model::{
    EU5_APP_ID, FORMAT_VERSION, Hierarchy, LocationId, LocationKind, LocationRecord, MapColor,
    RiverWidthMetadata, StoredDataset, SymbolId,
};

use super::{decode_blob, encode_blob, load_dataset, replace, write_dataset};

#[test]
fn blob_round_trip_is_deterministic() {
    let stored = fixture();
    let first = encode_blob(&stored);
    let second = encode_blob(&stored);
    assert!(first.is_ok());
    assert_eq!(first.as_ref().ok(), second.as_ref().ok());
    let decoded = second.ok().and_then(|bytes| decode_blob(&bytes).ok());
    assert_eq!(decoded.map(|value| value.stored), Some(stored));
}

#[test]
fn rejects_truncation_and_wrong_schema() {
    let encoded = encode_blob(&fixture()).ok();
    assert!(
        encoded
            .as_ref()
            .is_some_and(|bytes| decode_blob(&bytes[..bytes.len() / 2]).is_err())
    );
    let mut wrong = fixture();
    wrong.format_version += 1;
    assert!(encode_blob(&wrong).is_err());
}

#[test]
fn rejects_wrong_envelope_magic() {
    let Some(encoded) = encode_blob(&fixture()).ok() else {
        return;
    };
    let Some(mut expanded) = zstd::stream::decode_all(std::io::Cursor::new(encoded)).ok() else {
        return;
    };
    if let Some(first) = expanded.first_mut() {
        *first = b'X';
    }
    let Some(corrupt) = zstd::stream::encode_all(std::io::Cursor::new(expanded), 1).ok() else {
        return;
    };
    assert!(decode_blob(&corrupt).is_err());
}

#[test]
fn overwrite_requires_force_and_replacement_validates() {
    let directory = std::env::temp_dir().join(format!(
        "eu5-location-filter-storage-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&directory);
    assert!(fs::create_dir_all(&directory).is_ok());
    let path = directory.join("data.bitcode.zst");
    let stored = fixture();
    assert!(write_dataset(&path, &stored, false).is_ok());
    assert!(write_dataset(&path, &stored, false).is_err());
    assert!(write_dataset(&path, &stored, true).is_ok());
    assert!(load_dataset(&path).is_ok());
    let _ = fs::remove_dir_all(directory);
}

#[test]
fn failed_replacement_restores_existing_file() {
    let directory = std::env::temp_dir().join(format!(
        "eu5-location-filter-recovery-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&directory);
    assert!(fs::create_dir_all(&directory).is_ok());
    let target = directory.join("data.bitcode.zst");
    assert!(fs::write(&target, b"usable").is_ok());
    assert!(replace(&target, &directory.join("missing.tmp")).is_err());
    assert_eq!(fs::read(&target).ok(), Some(b"usable".to_vec()));
    let _ = fs::remove_dir_all(directory);
}

fn fixture() -> StoredDataset {
    StoredDataset {
        format_version: FORMAT_VERSION,
        app_id: EU5_APP_ID,
        build_id: 42,
        river_widths: RiverWidthMetadata {
            level_count: 3,
            width_min: 1.0,
            width_max: 2.0,
        },
        dictionary: vec![
            "one".to_owned(),
            "One".to_owned(),
            "flatland".to_owned(),
            "group".to_owned(),
        ],
        localizations: Vec::new(),
        locations: vec![LocationRecord {
            id: LocationId(0),
            key: SymbolId(0),
            name: SymbolId(1),
            kind: LocationKind::Land,
            color: MapColor(1),
            topography: SymbolId(2),
            vegetation: None,
            climate: None,
            religion: None,
            culture: None,
            raw_material: None,
            modifier: None,
            harbor_suitability: None,
            movement_assistance: None,
            hierarchy: Hierarchy {
                continent: SymbolId(3),
                subcontinent: SymbolId(3),
                region: SymbolId(3),
                area: SymbolId(3),
                province: SymbolId(3),
            },
            coastal: false,
            connected_sea: None,
            river: None,
            static_population_capacity: None,
        }],
        diagnostics: Vec::new(),
    }
}
