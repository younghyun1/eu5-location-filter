//! Envelope, naming, digest, and desktop file tests for language packs.

use super::{
    GameLanguage, MAX_PACK_COMPRESSED_SIZE, MAX_PACK_DECOMPRESSED_SIZE, PACK_MAGIC, decode_pack,
    dictionary_digest, pack_file_name,
};
use crate::AppError;

/// Compresses a hand-built envelope; level 1 keeps the oversized case fast.
fn frame(magic: &[u8], declared: u64, payload: &[u8]) -> Vec<u8> {
    let mut expanded = magic.to_vec();
    expanded.extend_from_slice(&declared.to_le_bytes());
    expanded.extend_from_slice(payload);
    zstd::bulk::compress(&expanded, 1).unwrap_or_default()
}

#[test]
fn english_has_no_pack_file_and_others_carry_the_build() {
    assert_eq!(pack_file_name(GameLanguage::English, 7), None);
    let korean = pack_file_name(GameLanguage::Korean, 26_547_371);
    assert_eq!(korean.as_deref(), Some("ko.26547371.bitcode.zst"));
    let chinese = pack_file_name(GameLanguage::SimplifiedChinese, 0);
    assert_eq!(chinese.as_deref(), Some("zh-hans.0.bitcode.zst"));
}

#[test]
fn digest_separates_entry_boundaries() {
    let joined = dictionary_digest(&["ab".to_owned(), "c".to_owned()]);
    let split = dictionary_digest(&["a".to_owned(), "bc".to_owned()]);
    assert_ne!(joined, split);
    assert_ne!(dictionary_digest(&[]), dictionary_digest(&[String::new()]));
}

#[test]
fn rejects_oversized_input_before_decompression() {
    let oversized = vec![0_u8; usize::try_from(MAX_PACK_COMPRESSED_SIZE + 1).unwrap_or(0)];
    assert!(matches!(decode_pack(&oversized), Err(AppError::InvalidData(_))));
}

#[test]
fn rejects_oversized_expansion() {
    let size = usize::try_from(MAX_PACK_DECOMPRESSED_SIZE + 1).unwrap_or(0);
    let compressed = zstd::bulk::compress(&vec![0_u8; size], 1).unwrap_or_default();
    assert!(!compressed.is_empty());
    assert!(matches!(decode_pack(&compressed), Err(AppError::InvalidData(_))));
}

#[test]
fn rejects_malformed_envelopes_by_family() {
    assert!(matches!(
        decode_pack(b"not a zstd frame"),
        Err(AppError::Compression(_))
    ));
    let wrong_magic = frame(b"EU5IDX\0\x01", 3, &[1, 2, 3]);
    assert!(matches!(decode_pack(&wrong_magic), Err(AppError::InvalidData(_))));
    let mut short = PACK_MAGIC.to_vec();
    short.extend_from_slice(&[3, 0]);
    let truncated_header = zstd::bulk::compress(&short, 1).unwrap_or_default();
    assert!(matches!(
        decode_pack(&truncated_header),
        Err(AppError::InvalidData(_))
    ));
    let wrong_length = frame(PACK_MAGIC, 4, &[1, 2, 3]);
    assert!(matches!(decode_pack(&wrong_length), Err(AppError::InvalidData(_))));
    let garbage = frame(PACK_MAGIC, 3, &[1, 2, 3]);
    assert!(matches!(decode_pack(&garbage), Err(AppError::Encoding(_))));
}

#[cfg(feature = "desktop")]
mod desktop {
    use std::fs;

    use super::super::{
        MAX_PACK_DECOMPRESSED_SIZE, decode_pack, encode_pack, load_pack, write_pack,
    };
    use crate::AppError;
    use crate::filter::test_fixture::{fixture, fixture_pack};

    #[test]
    fn envelope_round_trips_deterministically_within_the_window_limit() -> Result<(), AppError> {
        let pack = fixture_pack(&fixture());
        let first = encode_pack(&pack)?;
        assert_eq!(first, encode_pack(&pack)?);
        assert_eq!(decode_pack(&first)?, pack);
        // The browser decoder rejects frames whose window exceeds the decompressed limit, so
        // the frame header must carry the content size.
        let content = zstd::zstd_safe::get_frame_content_size(&first).ok().flatten();
        assert!(content.is_some_and(|size| size <= MAX_PACK_DECOMPRESSED_SIZE));
        Ok(())
    }

    #[test]
    fn truncated_frames_are_rejected() -> Result<(), AppError> {
        let encoded = encode_pack(&fixture_pack(&fixture()))?;
        let half = encoded.get(..encoded.len() / 2).unwrap_or_default();
        assert!(decode_pack(half).is_err());
        assert!(decode_pack(&[]).is_err());
        Ok(())
    }

    #[test]
    fn writes_require_force_and_remove_other_builds() -> Result<(), AppError> {
        let directory = std::env::temp_dir().join(format!(
            "eu5-location-filter-lang-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory)
            .map_err(|source| AppError::io("create", &directory, source))?;
        let stale = directory.join("ko.0.bitcode.zst");
        let other_language = directory.join("ja.0.bitcode.zst");
        let unrelated = directory.join("ko.0.bitcode.zst.bak");
        for path in [&stale, &other_language, &unrelated] {
            fs::write(path, b"old").map_err(|source| AppError::io("write", path, source))?;
        }
        let pack = fixture_pack(&fixture());
        let path = directory.join("ko.1.bitcode.zst");
        write_pack(&path, &pack, false)?;
        assert!(write_pack(&path, &pack, false).is_err());
        write_pack(&path, &pack, true)?;
        assert_eq!(load_pack(&path)?, pack);
        assert!(!stale.exists());
        assert!(other_language.exists());
        assert!(unrelated.exists());
        let missing = load_pack(&directory.join("fr.1.bitcode.zst"));
        assert!(missing.as_ref().is_err_and(AppError::is_retryable));
        assert!(matches!(missing, Err(AppError::Io { .. })));
        let _ = fs::remove_dir_all(directory);
        Ok(())
    }
}
