//! Pack envelope, file naming, pairing digest, and validation entry points.

#[cfg(feature = "desktop")]
use std::path::Path;

use super::{GameLanguage, StoredLanguagePack, ValidatedLanguagePack};
use crate::AppError;
use crate::model::Dataset;

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
    let _ = compressed;
    Err(not_implemented("decoding"))
}

/// Checks pairing, bounds, permutations, ranks, and postings against `dataset`.
///
/// Runs in linear time over the pack; the desktop loader calls it off the UI thread.
pub fn validate_pack(
    stored: StoredLanguagePack,
    dataset: &Dataset,
    expected: GameLanguage,
) -> Result<ValidatedLanguagePack, AppError> {
    let _ = (stored, dataset, expected);
    Err(not_implemented("validation"))
}

/// Encodes a pack as one level-22 zstd frame inside the `PACK_MAGIC` envelope.
#[cfg(feature = "desktop")]
pub fn encode_pack(pack: &StoredLanguagePack) -> Result<Vec<u8>, AppError> {
    let _ = pack;
    Err(not_implemented("encoding"))
}

/// Reads a pack file after enforcing `MAX_PACK_COMPRESSED_SIZE` on its metadata.
#[cfg(feature = "desktop")]
pub fn load_pack(path: &Path) -> Result<StoredLanguagePack, AppError> {
    let _ = path;
    Err(not_implemented("loading"))
}

/// Writes through a verified temporary file, then removes same-language packs of other builds
/// so the build script finds exactly one file per language.
#[cfg(feature = "desktop")]
pub fn write_pack(path: &Path, pack: &StoredLanguagePack, force: bool) -> Result<(), AppError> {
    let _ = (path, pack, force);
    Err(not_implemented("writing"))
}

/// Error returned by entry points whose bodies are not filled in yet.
pub(super) fn not_implemented(stage: &str) -> AppError {
    AppError::InvalidData(format!(
        "language pack {stage} is not implemented in this build"
    ))
}

#[cfg(test)]
mod tests {
    use super::{dictionary_digest, pack_file_name};
    use crate::lang_pack::GameLanguage;

    #[test]
    fn english_has_no_pack_file_and_others_carry_the_build() {
        assert_eq!(pack_file_name(GameLanguage::English, 7), None);
        let korean = pack_file_name(GameLanguage::Korean, 26_547_371);
        assert_eq!(korean.as_deref(), Some("ko.26547371.bitcode.zst"));
    }

    #[test]
    fn digest_separates_entry_boundaries() {
        let joined = dictionary_digest(&["ab".to_owned(), "c".to_owned()]);
        let split = dictionary_digest(&["a".to_owned(), "bc".to_owned()]);
        assert_ne!(joined, split);
    }
}
