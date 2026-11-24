//! Pack envelope, file naming, and the pairing digest.
//!
//! The envelope mirrors the location and index bundles: `PACK_MAGIC`, the payload length as a
//! little-endian `u64`, then the bitcode payload, all inside one zstd frame.

use super::{
    GameLanguage, MAX_PACK_COMPRESSED_SIZE, MAX_PACK_DECOMPRESSED_SIZE, PACK_MAGIC,
    StoredLanguagePack,
};
use crate::AppError;
use crate::compression::decode_limited;

// Validation and desktop file access live in sibling modules to keep each file small; these
// re-exports keep the frozen `codec::` paths working for code that imports them from here.
#[cfg(feature = "desktop")]
pub use super::files::{load_pack, write_pack};
pub use super::validate::validate_pack;

/// Magic plus the length field.
const HEADER_LEN: usize = 16;

/// File name of a pack for one dataset build; `None` for English, which has no pack file.
#[must_use]
pub fn pack_file_name(language: GameLanguage, build_id: u64) -> Option<String> {
    if language.is_embedded() {
        return None;
    }
    Some(format!("{}.{build_id}.bitcode.zst", language.code()))
}

/// FNV-1a 64 over each dictionary entry as a little-endian `u32` byte length plus its bytes.
///
/// This pairs a pack with dictionary content, not only with a build ID; it is not a security
/// measure, so a non-cryptographic hash is enough.
#[must_use]
pub fn dictionary_digest(dictionary: &[String]) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut hash = OFFSET_BASIS;
    for value in dictionary {
        let length = u32::try_from(value.len()).unwrap_or(u32::MAX);
        for byte in length.to_le_bytes().into_iter().chain(value.bytes()) {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(PRIME);
        }
    }
    hash
}

/// Decodes a pack envelope without pairing it.
///
/// Rejects frames over `MAX_PACK_COMPRESSED_SIZE`, expansions over
/// `MAX_PACK_DECOMPRESSED_SIZE`, a wrong magic, and a length header that disagrees with the
/// payload, before bitcode decoding runs.
pub fn decode_pack(compressed: &[u8]) -> Result<StoredLanguagePack, AppError> {
    if u64::try_from(compressed.len()).map_or(true, |length| length > MAX_PACK_COMPRESSED_SIZE) {
        return Err(AppError::InvalidData(
            "compressed language pack exceeds its size limit".to_owned(),
        ));
    }
    let expanded = decode_limited(compressed, MAX_PACK_DECOMPRESSED_SIZE)?;
    let payload = envelope_payload(&expanded)?;
    bitcode::decode::<StoredLanguagePack>(payload)
        .map_err(|error| AppError::Encoding(format!("language pack payload: {error}")))
}

/// Returns the payload after checking the magic and the declared length.
fn envelope_payload(expanded: &[u8]) -> Result<&[u8], AppError> {
    if expanded.get(..PACK_MAGIC.len()) != Some(PACK_MAGIC.as_slice()) {
        return Err(AppError::InvalidData(
            "language pack magic or envelope version is unsupported".to_owned(),
        ));
    }
    let length_bytes: Option<[u8; 8]> = expanded
        .get(PACK_MAGIC.len()..HEADER_LEN)
        .and_then(|bytes| bytes.try_into().ok());
    let Some(length_bytes) = length_bytes else {
        return Err(AppError::InvalidData(
            "language pack header is truncated".to_owned(),
        ));
    };
    let declared = u64::from_le_bytes(length_bytes);
    let Some(payload) = expanded.get(HEADER_LEN..) else {
        return Err(AppError::InvalidData(
            "language pack payload is truncated".to_owned(),
        ));
    };
    if u64::try_from(payload.len()).ok() != Some(declared) {
        return Err(AppError::InvalidData(
            "language pack payload length does not match its header".to_owned(),
        ));
    }
    Ok(payload)
}

/// Encodes a pack as one level-22 zstd frame inside the `PACK_MAGIC` envelope.
#[cfg(feature = "desktop")]
pub fn encode_pack(pack: &StoredLanguagePack) -> Result<Vec<u8>, AppError> {
    let payload = bitcode::encode(pack);
    let length = u64::try_from(payload.len())
        .map_err(|error| AppError::Encoding(format!("language pack length overflow: {error}")))?;
    if length > MAX_PACK_DECOMPRESSED_SIZE.saturating_sub(HEADER_LEN as u64) {
        return Err(AppError::InvalidData(
            "language pack exceeds the decompressed size limit".to_owned(),
        ));
    }
    let mut expanded = Vec::with_capacity(payload.len().saturating_add(HEADER_LEN));
    expanded.extend_from_slice(PACK_MAGIC);
    expanded.extend_from_slice(&length.to_le_bytes());
    expanded.extend_from_slice(&payload);
    // One-shot compression records the content size in the frame header, so the frame's window
    // never exceeds the payload. The browser decoder caps the window at the decompressed limit;
    // a streamed level-22 frame, as the index bundle uses, declares a 128 MiB window and would
    // be rejected at this smaller limit.
    let compressed = zstd::bulk::compress(&expanded, 22)
        .map_err(|error| AppError::Compression(error.to_string()))?;
    if u64::try_from(compressed.len()).map_or(true, |size| size > MAX_PACK_COMPRESSED_SIZE) {
        return Err(AppError::InvalidData(
            "compressed language pack exceeds its size limit".to_owned(),
        ));
    }
    Ok(compressed)
}

/// Error for builder entry points that are not filled in yet.
#[cfg(feature = "desktop")]
#[allow(
    dead_code,
    reason = "kept for builder bodies that may still return it; remove once none do"
)]
pub(super) fn not_implemented(stage: &str) -> AppError {
    AppError::InvalidData(format!(
        "language pack {stage} is not implemented in this build"
    ))
}
