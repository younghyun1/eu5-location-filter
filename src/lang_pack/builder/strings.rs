//! The pack string table and the diagnostics counted while a pack is built.

use std::collections::HashMap;

use crate::AppError;
use crate::lang_pack::{MAX_PACK_STRING_BYTES, PackStringId};
use crate::model::{ImportDiagnostic, MAX_DICTIONARY_BYTES};

/// Distinct pack strings in first-use order, bounded like the dataset dictionary.
#[derive(Default)]
pub(super) struct PackStrings {
    values: Vec<String>,
    lookup: HashMap<String, PackStringId>,
    bytes: usize,
}

impl PackStrings {
    /// Returns the ID of `value`, adding it on first use.
    pub(super) fn intern(&mut self, value: &str) -> Result<PackStringId, AppError> {
        if let Some(id) = self.lookup.get(value) {
            return Ok(*id);
        }
        let bytes = self.bytes.saturating_add(value.len());
        if bytes > MAX_DICTIONARY_BYTES {
            return Err(AppError::InvalidData(
                "language pack strings exceed the dictionary size limit".to_owned(),
            ));
        }
        let id = PackStringId(u32::try_from(self.values.len()).map_err(|error| {
            AppError::InvalidData(format!("language pack string index overflow: {error}"))
        })?);
        self.bytes = bytes;
        self.values.push(value.to_owned());
        self.lookup.insert(value.to_owned(), id);
        Ok(id)
    }

    /// The table in ID order.
    pub(super) fn into_values(self) -> Vec<String> {
        self.values
    }
}

/// Non-fatal findings of one build, reported as pack diagnostics when non-zero.
#[derive(Default)]
pub(super) struct Tally {
    /// Locations without a name in this language; they keep the English name.
    pub(super) missing_names: u32,
    /// Labelled symbols without a label in this language; they keep the English label.
    pub(super) missing_labels: u32,
    /// Values left with a literal `$key$` because the reference did not resolve.
    unresolved_references: u32,
    /// Values rejected because they are empty, too long, or contain NUL.
    rejected_values: u32,
    /// Russian culture labels shipped as adjective stems the game completes at runtime.
    culture_label_stems: u32,
}

impl Tally {
    /// Returns a value usable as pack text, counting one that has to fall back to English.
    ///
    /// Blank values count as missing at the call site (`None` is returned for them, and for
    /// absent keys); oversized or NUL-containing values also count as rejected here.
    pub(super) fn accept<'a>(&mut self, value: Option<&'a String>) -> Option<&'a str> {
        let value = value?.as_str();
        if value.trim().is_empty() {
            return None;
        }
        if value.len() > MAX_PACK_STRING_BYTES || value.contains('\0') {
            self.rejected_values += 1;
            return None;
        }
        Some(value)
    }

    /// Counts a value that still holds a `$key$` token after expansion.
    pub(super) fn check_references(&mut self, value: &str) {
        if has_unresolved_reference(value) {
            self.unresolved_references += 1;
        }
    }

    /// Counts a culture label that ends in a Russian adjective stem.
    pub(super) fn check_stem(&mut self, value: &str) {
        if value.ends_with("ск") || value.ends_with("цк") {
            self.culture_label_stems += 1;
        }
    }

    /// Non-zero counts in a fixed order, so the diagnostics are deterministic.
    pub(super) fn into_diagnostics(self) -> Vec<ImportDiagnostic> {
        [
            ("missing_location_name", self.missing_names),
            ("missing_label", self.missing_labels),
            ("unresolved_reference", self.unresolved_references),
            ("rejected_value", self.rejected_values),
            ("culture_label_stem", self.culture_label_stems),
        ]
        .into_iter()
        .filter(|(_, count)| *count > 0)
        .map(|(code, count)| ImportDiagnostic {
            code: code.to_owned(),
            count,
        })
        .collect()
    }
}

/// Whether `value` contains `$key$` with a non-empty key made of identifier characters, the
/// form `read_localizations` leaves in place when a reference does not resolve.
pub(super) fn has_unresolved_reference(value: &str) -> bool {
    let mut rest = value;
    while let Some(start) = rest.find('$') {
        let after = &rest[start + 1..];
        let Some(end) = after.find('$') else {
            return false;
        };
        let token = &after[..end];
        let is_key = !token.is_empty()
            && token
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || "_.|".contains(character));
        if is_key {
            return true;
        }
        rest = &after[end..];
    }
    false
}
