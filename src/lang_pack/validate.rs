//! Pack validation: schema, pairing, text bounds, and the language section, in contract order.
//!
//! Every check is linear in the pack and the dataset. Pairing failures return
//! `AppError::PackMismatch` so the interface can explain a stale download without matching
//! message text; every other failure is `AppError::InvalidData`.

use std::collections::HashSet;

use super::{
    GameLanguage, MAX_PACK_STRING_BYTES, PACK_FORMAT_VERSION, PackStringId, StoredLanguagePack,
    ValidatedLanguagePack, dictionary_digest, resolve_text,
};
use crate::AppError;
use crate::model::{Dataset, EU5_APP_ID, MAX_DICTIONARY_BYTES, MAX_SYMBOLS};

/// Most diagnostics a pack may carry; the importer records a handful.
const MAX_DIAGNOSTICS: usize = 64;
/// Longest diagnostic code in bytes.
const MAX_DIAGNOSTIC_CODE_BYTES: usize = 64;

/// Checks pairing, bounds, permutations, ranks, and postings against `dataset`.
///
/// Runs in linear time over the pack; the desktop loader calls it off the UI thread.
pub fn validate_pack(
    stored: StoredLanguagePack,
    dataset: &Dataset,
    expected: GameLanguage,
) -> Result<ValidatedLanguagePack, AppError> {
    let digest = dictionary_digest(&dataset.stored.dictionary);
    validate_paired(stored, dataset, expected, digest)
}

/// [`validate_pack`] with the dataset's dictionary digest already computed.
pub(crate) fn validate_paired(
    stored: StoredLanguagePack,
    dataset: &Dataset,
    expected: GameLanguage,
    digest: u64,
) -> Result<ValidatedLanguagePack, AppError> {
    if stored.format_version != PACK_FORMAT_VERSION || stored.app_id != EU5_APP_ID {
        return Err(AppError::InvalidData(
            "language pack schema or application ID is unsupported".to_owned(),
        ));
    }
    check_pairing(&stored, dataset, expected, digest)?;
    check_strings(&stored)?;
    check_references(&stored)?;
    check_diagnostics(&stored)?;
    {
        // The resolved text borrows the pack, so it must end before the pack moves below.
        let text = resolve_text(dataset, Some(&stored));
        crate::filter::validate_language_section(&stored.section, dataset, Some(&text))?;
    }
    let label_by_symbol = dense_labels(&stored)?;
    Ok(ValidatedLanguagePack {
        stored,
        label_by_symbol,
    })
}

/// Steps 2 and 3: language, build, counts, and dictionary content.
fn check_pairing(
    stored: &StoredLanguagePack,
    dataset: &Dataset,
    expected: GameLanguage,
    digest: u64,
) -> Result<(), AppError> {
    if stored.language.is_embedded() || stored.language != expected {
        return Err(AppError::PackMismatch(format!(
            "pack language {} was requested as {}",
            stored.language.code(),
            expected.code()
        )));
    }
    if stored.build_id != dataset.stored.build_id {
        return Err(AppError::PackMismatch(format!(
            "pack build {} differs from data build {}",
            stored.build_id, dataset.stored.build_id
        )));
    }
    if usize::try_from(stored.location_count).ok() != Some(dataset.stored.locations.len())
        || usize::try_from(stored.symbol_count).ok() != Some(dataset.stored.dictionary.len())
    {
        return Err(AppError::PackMismatch(
            "pack location or symbol count differs from the data".to_owned(),
        ));
    }
    if stored.dictionary_digest != digest {
        return Err(AppError::PackMismatch(
            "pack was built for different dictionary content".to_owned(),
        ));
    }
    Ok(())
}

/// Step 4: count, length, NUL, uniqueness, and total size of the string table.
fn check_strings(stored: &StoredLanguagePack) -> Result<(), AppError> {
    let limit = usize::try_from(stored.location_count)
        .ok()
        .zip(usize::try_from(stored.symbol_count).ok())
        .map(|(locations, symbols)| locations.saturating_add(symbols));
    if limit.is_none_or(|limit| stored.strings.len() > limit) {
        return Err(AppError::InvalidData(
            "language pack has more strings than names and labels".to_owned(),
        ));
    }
    let mut seen = HashSet::with_capacity(stored.strings.len());
    let mut total = 0_usize;
    for value in &stored.strings {
        total = total.saturating_add(value.len());
        if value.is_empty()
            || value.len() > MAX_PACK_STRING_BYTES
            || value.contains('\0')
            || !seen.insert(value.as_str())
            || total > MAX_DICTIONARY_BYTES
        {
            return Err(AppError::InvalidData(
                "language pack string is empty, oversized, duplicated, or contains NUL".to_owned(),
            ));
        }
    }
    Ok(())
}

/// Steps 5 and 6: one name slot per location and ordered, in-range labels.
fn check_references(stored: &StoredLanguagePack) -> Result<(), AppError> {
    if usize::try_from(stored.location_count).ok() != Some(stored.location_names.len()) {
        return Err(AppError::InvalidData(
            "language pack name table does not cover every location".to_owned(),
        ));
    }
    let strings = stored.strings.len();
    if stored
        .location_names
        .iter()
        .flatten()
        .any(|value| !in_range(*value, strings))
    {
        return Err(AppError::InvalidData(
            "language pack name refers to a missing string".to_owned(),
        ));
    }
    let ordered = stored
        .labels
        .windows(2)
        .all(|pair| matches!(pair, [left, right] if left.key < right.key));
    if usize::try_from(stored.symbol_count).map_or(true, |count| stored.labels.len() > count)
        || !ordered
        || stored
            .labels
            .iter()
            .any(|label| label.key.0 >= stored.symbol_count || !in_range(label.value, strings))
    {
        return Err(AppError::InvalidData(
            "language pack labels are unordered, duplicated, or out of range".to_owned(),
        ));
    }
    Ok(())
}

/// Step 7: diagnostics stay small, so a hostile pack cannot grow memory through them.
fn check_diagnostics(stored: &StoredLanguagePack) -> Result<(), AppError> {
    if stored.diagnostics.len() > MAX_DIAGNOSTICS
        || stored
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code.len() > MAX_DIAGNOSTIC_CODE_BYTES)
    {
        return Err(AppError::InvalidData(
            "language pack diagnostics exceed their limits".to_owned(),
        ));
    }
    Ok(())
}

/// Step 9: the dense symbol-to-label table, so a cell lookup is one index.
fn dense_labels(stored: &StoredLanguagePack) -> Result<Vec<Option<PackStringId>>, AppError> {
    let count = usize::try_from(stored.symbol_count)
        .ok()
        .filter(|count| *count <= MAX_SYMBOLS);
    let Some(count) = count else {
        return Err(AppError::InvalidData(
            "language pack symbol count exceeds its limit".to_owned(),
        ));
    };
    let mut table = vec![None; count];
    for label in &stored.labels {
        let slot = usize::try_from(label.key.0)
            .ok()
            .and_then(|slot| table.get_mut(slot));
        let Some(slot) = slot else {
            return Err(AppError::InvalidData(
                "language pack label is out of range".to_owned(),
            ));
        };
        *slot = Some(label.value);
    }
    Ok(table)
}

fn in_range(value: PackStringId, strings: usize) -> bool {
    usize::try_from(value.0).is_ok_and(|slot| slot < strings)
}
