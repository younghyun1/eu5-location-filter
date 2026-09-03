//! Versioned compressed blob encoding and crash-safe replacement.

use std::collections::{HashMap, HashSet};
use std::fs::{self, OpenOptions};
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};

use crate::AppError;
use crate::model::{
    Dataset, EU5_APP_ID, FORMAT_VERSION, LocationRecord, MAX_DICTIONARY_BYTES, MAX_LOCATIONS,
    MAX_SYMBOLS, StoredDataset, SymbolId,
};

const MAGIC: &[u8; 8] = b"EU5LOC\0\x01";
const MAX_COMPRESSED_SIZE: u64 = 256 * 1024 * 1024;
const MAX_DECOMPRESSED_SIZE: u64 = 128 * 1024 * 1024;

/// Encodes a deterministic zstd frame containing the versioned bitcode payload.
pub fn encode_blob(stored: &StoredDataset) -> Result<Vec<u8>, AppError> {
    validate_stored(stored)?;
    let payload = bitcode::encode(stored);
    let payload_len = u64::try_from(payload.len())
        .map_err(|error| AppError::Encoding(format!("payload length overflow: {error}")))?;
    if payload_len > MAX_DECOMPRESSED_SIZE.saturating_sub(16) {
        return Err(AppError::InvalidData(
            "encoded dataset exceeds decompressed size limit".to_owned(),
        ));
    }
    let mut frame_input = Vec::with_capacity(payload.len().saturating_add(16));
    frame_input.extend_from_slice(MAGIC);
    frame_input.extend_from_slice(&payload_len.to_le_bytes());
    frame_input.extend_from_slice(&payload);
    let compressed = zstd::stream::encode_all(Cursor::new(frame_input), 9)
        .map_err(|error| AppError::Compression(error.to_string()))?;
    if compressed.len() as u64 > MAX_COMPRESSED_SIZE {
        return Err(AppError::InvalidData(
            "compressed dataset exceeds size limit".to_owned(),
        ));
    }
    Ok(compressed)
}

/// Decodes and validates a compressed dataset from memory.
pub fn decode_blob(compressed: &[u8]) -> Result<Dataset, AppError> {
    if compressed.len() as u64 > MAX_COMPRESSED_SIZE {
        return Err(AppError::InvalidData(
            "compressed dataset exceeds size limit".to_owned(),
        ));
    }
    let decoder = zstd::stream::read::Decoder::new(Cursor::new(compressed))
        .map_err(|error| AppError::Compression(error.to_string()))?;
    let mut expanded = Vec::new();
    decoder
        .take(MAX_DECOMPRESSED_SIZE + 1)
        .read_to_end(&mut expanded)
        .map_err(|error| AppError::Compression(error.to_string()))?;
    if expanded.len() as u64 > MAX_DECOMPRESSED_SIZE {
        return Err(AppError::InvalidData(
            "decompressed dataset exceeds size limit".to_owned(),
        ));
    }
    if expanded.get(..MAGIC.len()) != Some(MAGIC.as_slice()) {
        return Err(AppError::InvalidData(
            "dataset magic or envelope version is unsupported".to_owned(),
        ));
    }
    let length_bytes: [u8; 8] = expanded
        .get(8..16)
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or_else(|| AppError::InvalidData("dataset length header is truncated".to_owned()))?;
    let declared = usize::try_from(u64::from_le_bytes(length_bytes))
        .map_err(|error| AppError::InvalidData(format!("dataset length overflow: {error}")))?;
    let payload = expanded
        .get(16..)
        .ok_or_else(|| AppError::InvalidData("dataset payload is truncated".to_owned()))?;
    if payload.len() != declared {
        return Err(AppError::InvalidData(format!(
            "dataset payload length is {}; header declares {declared}",
            payload.len()
        )));
    }
    let stored = bitcode::decode::<StoredDataset>(payload)
        .map_err(|error| AppError::Encoding(error.to_string()))?;
    build_dataset(stored)
}

/// Loads a compressed dataset while enforcing the on-disk size limit first.
pub fn load_dataset(path: &Path) -> Result<Dataset, AppError> {
    let metadata = fs::metadata(path).map_err(|source| AppError::io("inspect", path, source))?;
    if metadata.len() > MAX_COMPRESSED_SIZE {
        return Err(AppError::InvalidData(format!(
            "data file exceeds {} bytes",
            MAX_COMPRESSED_SIZE
        )));
    }
    let compressed = fs::read(path).map_err(|source| AppError::io("read", path, source))?;
    decode_blob(&compressed)
}

/// Writes through a validated same-directory temporary file and safely replaces on force.
pub fn write_dataset(path: &Path, stored: &StoredDataset, force: bool) -> Result<(), AppError> {
    if path.exists() && !force {
        return Err(AppError::InvalidData(format!(
            "data file already exists; pass --force to replace it: {}",
            path.display()
        )));
    }
    let parent = path
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|source| AppError::io("create data directory", parent, source))?;
    let compressed = encode_blob(stored)?;
    let temp = temporary_path(path);
    remove_if_exists(&temp)?;
    write_and_sync(&temp, &compressed)?;
    if let Err(error) = load_dataset(&temp) {
        let _ = fs::remove_file(&temp);
        return Err(error);
    }
    replace(path, &temp)
}

fn build_dataset(stored: StoredDataset) -> Result<Dataset, AppError> {
    validate_stored(&stored)?;
    let mut by_key = HashMap::with_capacity(stored.locations.len());
    let mut by_color = HashMap::with_capacity(stored.locations.len());
    for record in &stored.locations {
        if by_key.insert(record.key, record.id).is_some() {
            return Err(AppError::InvalidData(
                "duplicate internal identifier in blob".to_owned(),
            ));
        }
        if by_color.insert(record.color, record.id).is_some() {
            return Err(AppError::InvalidData(format!(
                "duplicate color {} in blob",
                record.color.hex()
            )));
        }
    }
    let mut localized = HashMap::with_capacity(stored.localizations.len());
    for value in &stored.localizations {
        if localized.insert(value.key, value.value).is_some() {
            return Err(AppError::InvalidData(
                "duplicate localization key in blob".to_owned(),
            ));
        }
    }
    Ok(Dataset {
        stored,
        by_key,
        by_color,
        localized,
    })
}

fn validate_stored(stored: &StoredDataset) -> Result<(), AppError> {
    if stored.format_version != FORMAT_VERSION {
        return Err(AppError::InvalidData(format!(
            "unsupported schema version {}",
            stored.format_version
        )));
    }
    if stored.app_id != EU5_APP_ID {
        return Err(AppError::InvalidData(format!(
            "unexpected Steam app ID {}",
            stored.app_id
        )));
    }
    if stored.dictionary.len() > MAX_SYMBOLS
        || stored.dictionary.iter().map(String::len).sum::<usize>() > MAX_DICTIONARY_BYTES
    {
        return Err(AppError::InvalidData(
            "dictionary exceeds configured limits".to_owned(),
        ));
    }
    if stored.locations.len() > MAX_LOCATIONS || stored.locations.is_empty() {
        return Err(AppError::InvalidData(
            "location count is outside configured limits".to_owned(),
        ));
    }
    let symbol_count = stored.dictionary.len();
    let location_count = stored.locations.len();
    let mut keys = HashSet::with_capacity(location_count);
    let mut colors = HashSet::with_capacity(location_count);
    for (index, record) in stored.locations.iter().enumerate() {
        validate_record(
            record,
            index,
            symbol_count,
            location_count,
            stored.river_widths.level_count,
        )?;
        if !keys.insert(record.key) || !colors.insert(record.color) {
            return Err(AppError::InvalidData(
                "location identifiers and colors must be unique".to_owned(),
            ));
        }
    }
    for value in &stored.localizations {
        validate_symbol(value.key, symbol_count)?;
        validate_symbol(value.value, symbol_count)?;
    }
    if stored.river_widths.level_count == 0
        || !stored.river_widths.width_min.is_finite()
        || !stored.river_widths.width_max.is_finite()
        || stored.river_widths.width_min <= 0.0
        || stored.river_widths.width_max < stored.river_widths.width_min
    {
        return Err(AppError::InvalidData(
            "river width metadata is invalid".to_owned(),
        ));
    }
    Ok(())
}

fn validate_record(
    record: &LocationRecord,
    index: usize,
    symbol_count: usize,
    location_count: usize,
    max_river_level: u8,
) -> Result<(), AppError> {
    if usize::try_from(record.id.0).ok() != Some(index) {
        return Err(AppError::InvalidData(
            "location IDs must match storage order".to_owned(),
        ));
    }
    for symbol in [
        Some(record.key),
        Some(record.name),
        Some(record.topography),
        record.vegetation,
        record.climate,
        record.religion,
        record.culture,
        record.raw_material,
        record.modifier,
        Some(record.hierarchy.continent),
        Some(record.hierarchy.subcontinent),
        Some(record.hierarchy.region),
        Some(record.hierarchy.area),
        Some(record.hierarchy.province),
    ]
    .into_iter()
    .flatten()
    {
        validate_symbol(symbol, symbol_count)?;
    }
    if record
        .harbor_suitability
        .is_some_and(|value| !value.is_finite())
        || record
            .movement_assistance
            .is_some_and(|value| value.into_iter().any(|component| !component.is_finite()))
    {
        return Err(AppError::InvalidData(
            "location contains a non-finite number".to_owned(),
        ));
    }
    if record
        .connected_sea
        .is_some_and(|value| usize::try_from(value.0).map_or(true, |value| value >= location_count))
    {
        return Err(AppError::InvalidData(
            "connected sea location is out of range".to_owned(),
        ));
    }
    if record.river.is_some_and(|river| {
        river.level.0 > max_river_level
            || !river.rendered_width.is_finite()
            || river.rendered_width < 0.0
    }) {
        return Err(AppError::InvalidData("river data is invalid".to_owned()));
    }
    Ok(())
}

fn validate_symbol(symbol: SymbolId, count: usize) -> Result<(), AppError> {
    if usize::try_from(symbol.0).map_or(true, |value| value >= count) {
        return Err(AppError::InvalidData(
            "dictionary symbol is out of range".to_owned(),
        ));
    }
    Ok(())
}

fn write_and_sync(path: &Path, bytes: &[u8]) -> Result<(), AppError> {
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|source| AppError::io("create temporary data file", path, source))?;
    file.write_all(bytes)
        .map_err(|source| AppError::io("write temporary data file", path, source))?;
    file.flush()
        .and_then(|()| file.sync_all())
        .map_err(|source| AppError::io("flush temporary data file", path, source))
}

fn replace(target: &Path, temp: &Path) -> Result<(), AppError> {
    if !target.exists() {
        return fs::rename(temp, target)
            .map_err(|source| AppError::io("install data file", target, source));
    }
    let backup = backup_path(target);
    remove_if_exists(&backup)?;
    fs::rename(target, &backup)
        .map_err(|source| AppError::io("back up data file", target, source))?;
    if let Err(source) = fs::rename(temp, target) {
        let restore_result = fs::rename(&backup, target);
        let _ = fs::remove_file(temp);
        return match restore_result {
            Ok(()) => Err(AppError::io(
                "install replacement data file",
                target,
                source,
            )),
            Err(restore) => Err(AppError::InvalidData(format!(
                "replacement failed ({source}) and backup restore failed ({restore}); backup remains at {}",
                backup.display()
            ))),
        };
    }
    load_dataset(target)?;
    remove_if_exists(&backup)
}

fn temporary_path(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map_or_else(|| "eu5-locations".into(), |value| value.to_os_string());
    let mut temp_name = name;
    temp_name.push(format!(".{}.tmp", std::process::id()));
    path.with_file_name(temp_name)
}

fn backup_path(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map_or_else(|| "eu5-locations".into(), |value| value.to_os_string());
    let mut backup_name = name;
    backup_name.push(".bak");
    path.with_file_name(backup_name)
}

fn remove_if_exists(path: &Path) -> Result<(), AppError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(AppError::io("remove stale data file", path, source)),
    }
}

#[cfg(test)]
mod tests;
