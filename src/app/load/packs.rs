//! Language pack decoding and validation shared by both targets: off the UI thread on desktop,
//! inside update in the browser.

use crate::AppError;
use crate::lang_pack::{GameLanguage, ValidatedLanguagePack, decode_pack, validate_pack};
use crate::model::Dataset;

/// Decodes compressed pack bytes and validates them against the dataset for `expected`.
/// Pairing failures are `AppError::PackMismatch`, structural ones `InvalidData`, as classified
/// in the language pack contract.
pub(crate) fn validate_pack_bytes(
    bytes: &[u8],
    dataset: &Dataset,
    expected: GameLanguage,
) -> Result<ValidatedLanguagePack, AppError> {
    let stored = decode_pack(bytes)?;
    validate_pack(stored, dataset, expected)
}
