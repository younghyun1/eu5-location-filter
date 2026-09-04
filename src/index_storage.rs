//! Level-22 bitcode-zstd storage for precomputed search and sort indexes.

#[cfg(feature = "desktop")]
use std::fs::{self, OpenOptions};
#[cfg(feature = "desktop")]
use std::io::Cursor;
#[cfg(feature = "desktop")]
use std::io::Write;
#[cfg(feature = "desktop")]
use std::path::{Path, PathBuf};

use crate::AppError;
use crate::compression::decode_limited;
use crate::filter::StoredFilterIndex;

const MAGIC: &[u8; 8] = b"EU5IDX\0\x01";
const MAX_COMPRESSED_SIZE: u64 = 128 * 1024 * 1024;
const MAX_DECOMPRESSED_SIZE: u64 = 128 * 1024 * 1024;

/// Encodes a deterministic index payload as one maximum-level zstd frame.
#[cfg(feature = "desktop")]
pub fn encode_index(index: &StoredFilterIndex) -> Result<Vec<u8>, AppError> {
    let payload = bitcode::encode(index);
    let length = u64::try_from(payload.len())
        .map_err(|error| AppError::Encoding(format!("index length overflow: {error}")))?;
    if length > MAX_DECOMPRESSED_SIZE.saturating_sub(16) {
        return Err(AppError::InvalidData(
            "filter index exceeds the decompressed size limit".to_owned(),
        ));
    }
    let mut expanded = Vec::with_capacity(payload.len().saturating_add(16));
    expanded.extend_from_slice(MAGIC);
    expanded.extend_from_slice(&length.to_le_bytes());
    expanded.extend_from_slice(&payload);
    let compressed = zstd::stream::encode_all(Cursor::new(expanded), 22)
        .map_err(|error| AppError::Compression(error.to_string()))?;
    if compressed.len() as u64 > MAX_COMPRESSED_SIZE {
        return Err(AppError::InvalidData(
            "compressed filter index exceeds its size limit".to_owned(),
        ));
    }
    Ok(compressed)
}

/// Decodes an index envelope; dataset pairing is validated by `FilterEngine`.
pub fn decode_index(compressed: &[u8]) -> Result<StoredFilterIndex, AppError> {
    if compressed.len() as u64 > MAX_COMPRESSED_SIZE {
        return Err(AppError::InvalidData(
            "compressed filter index exceeds its size limit".to_owned(),
        ));
    }
    let expanded = decode_limited(compressed, MAX_DECOMPRESSED_SIZE)?;
    if expanded.get(..8) != Some(MAGIC.as_slice()) {
        return Err(AppError::InvalidData(
            "filter index magic or envelope version is unsupported".to_owned(),
        ));
    }
    let length_bytes: [u8; 8] = expanded
        .get(8..16)
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or_else(|| AppError::InvalidData("filter index header is truncated".to_owned()))?;
    let declared = usize::try_from(u64::from_le_bytes(length_bytes))
        .map_err(|error| AppError::InvalidData(format!("index length overflow: {error}")))?;
    let payload = expanded
        .get(16..)
        .ok_or_else(|| AppError::InvalidData("filter index payload is truncated".to_owned()))?;
    if payload.len() != declared {
        return Err(AppError::InvalidData(
            "filter index payload length does not match its header".to_owned(),
        ));
    }
    bitcode::decode(payload).map_err(|error| AppError::Encoding(error.to_string()))
}

/// Reads and decodes an external index bundle.
#[cfg(feature = "desktop")]
pub fn load_index(path: &Path) -> Result<StoredFilterIndex, AppError> {
    let size = fs::metadata(path)
        .map_err(|source| AppError::io("inspect", path, source))?
        .len();
    if size > MAX_COMPRESSED_SIZE {
        return Err(AppError::InvalidData(
            "filter index file exceeds its size limit".to_owned(),
        ));
    }
    let bytes = fs::read(path).map_err(|source| AppError::io("read", path, source))?;
    decode_index(&bytes)
}

/// Writes a validated temporary index before replacing the prior bundle.
#[cfg(feature = "desktop")]
pub fn write_index(path: &Path, index: &StoredFilterIndex, force: bool) -> Result<(), AppError> {
    if path.exists() && !force {
        return Err(AppError::InvalidData(format!(
            "index file already exists; pass --force to replace it: {}",
            path.display()
        )));
    }
    let parent = path
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|source| AppError::io("create index directory", parent, source))?;
    let bytes = encode_index(index)?;
    let temp = temporary_path(path);
    remove_if_exists(&temp)?;
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temp)
        .map_err(|source| AppError::io("create temporary index", &temp, source))?;
    file.write_all(&bytes)
        .and_then(|()| file.flush())
        .and_then(|()| file.sync_all())
        .map_err(|source| AppError::io("write temporary index", &temp, source))?;
    drop(file);
    decode_index(&fs::read(&temp).map_err(|source| AppError::io("read", &temp, source))?)?;
    replace(path, &temp)
}

#[cfg(feature = "desktop")]
fn replace(path: &Path, temp: &Path) -> Result<(), AppError> {
    let backup = backup_path(path);
    remove_if_exists(&backup)?;
    if path.exists() {
        fs::rename(path, &backup)
            .map_err(|source| AppError::io("back up filter index", path, source))?;
    }
    if let Err(source) = fs::rename(temp, path) {
        if backup.exists() {
            let _ = fs::rename(&backup, path);
        }
        return Err(AppError::io("install filter index", path, source));
    }
    remove_if_exists(&backup)
}

#[cfg(feature = "desktop")]
fn temporary_path(path: &Path) -> PathBuf {
    suffixed(path, &format!(".{}.tmp", std::process::id()))
}

#[cfg(feature = "desktop")]
fn backup_path(path: &Path) -> PathBuf {
    suffixed(path, ".bak")
}

#[cfg(feature = "desktop")]
fn suffixed(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path
        .file_name()
        .map_or_else(|| "eu5-indexes".into(), |value| value.to_os_string());
    name.push(suffix);
    path.with_file_name(name)
}

#[cfg(feature = "desktop")]
fn remove_if_exists(path: &Path) -> Result<(), AppError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(AppError::io("remove stale index file", path, source)),
    }
}

#[cfg(all(test, feature = "desktop"))]
mod tests {
    use super::{decode_index, encode_index};
    use crate::filter::StoredFilterIndex;

    #[test]
    fn index_envelope_round_trips() {
        let index = StoredFilterIndex {
            format_version: 1,
            app_id: crate::model::EU5_APP_ID,
            build_id: 42,
            location_count: 0,
            searchable: Vec::new(),
            orders: Vec::new(),
        };
        let encoded = encode_index(&index);
        assert!(encoded.is_ok());
        assert_eq!(
            encoded.ok().and_then(|bytes| decode_index(&bytes).ok()),
            Some(index)
        );
    }
}
