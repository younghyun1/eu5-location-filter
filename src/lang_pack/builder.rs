//! Offline pack construction from an EU5 install and a validated dataset.
//!
//! Output is deterministic: names follow location IDs, labels follow symbol IDs, and strings are
//! interned in that order, so two builds from the same inputs encode to identical bytes. The
//! localization map is only ever looked up, never iterated.

mod strings;
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, HashMap, HashSet};

use super::{
    GameLanguage, LabelField, PACK_FORMAT_VERSION, PackLabel, StoredLanguagePack,
    dictionary_digest, resolve_text,
};
use crate::AppError;
use crate::import::record_use;
use crate::model::{Dataset, EU5_APP_ID, SymbolId};
use crate::parser::read_localizations;
use crate::steam::GameInstallation;
use strings::{PackStrings, Tally};

/// Localization keys a pack needs: every location key plus each labelled field's key.
///
/// Fails when one symbol is used both as a static modifier and in another labelled field,
/// because the two fields resolve labels through different localization keys.
pub fn requested_keys(dataset: &Dataset) -> Result<HashSet<String>, AppError> {
    let labelled = labelled_symbols(dataset)?;
    let mut keys = HashSet::with_capacity(dataset.stored.locations.len() + labelled.len());
    for record in &dataset.stored.locations {
        keys.insert(identifier(dataset, record.key)?.to_owned());
    }
    for (symbol, field) in &labelled {
        let key = field.localization_key(identifier(dataset, *symbol)?);
        keys.insert(key.into_owned());
    }
    Ok(keys)
}

/// Builds a pack from resolved localization values without touching the filesystem.
///
/// `localization` maps localization keys to values with `$key$` references already expanded
/// in the same language, as `parser::read_localizations` returns them.
pub fn build_pack(
    dataset: &Dataset,
    language: GameLanguage,
    localization: &HashMap<String, String>,
) -> Result<StoredLanguagePack, AppError> {
    if language.is_embedded() {
        return Err(AppError::InvalidData(
            "English text lives in the main bundles and has no pack".to_owned(),
        ));
    }
    let labelled = labelled_symbols(dataset)?;
    let mut strings = PackStrings::default();
    let mut tally = Tally::default();
    let mut location_names = Vec::with_capacity(dataset.stored.locations.len());
    for record in &dataset.stored.locations {
        let key = identifier(dataset, record.key)?;
        let english = dataset.symbol(record.name).unwrap_or_default();
        let name = match tally.accept(localization.get(key)) {
            Some(text) => {
                tally.check_references(text);
                text
            }
            None => {
                tally.missing_names += 1;
                english
            }
        };
        let slot = if name == english {
            None
        } else {
            Some(strings.intern(name)?)
        };
        location_names.push(slot);
    }
    let mut labels = Vec::new();
    for (&symbol, &field) in &labelled {
        let raw = identifier(dataset, symbol)?;
        let english = dataset.label(symbol).unwrap_or(raw);
        let key = field.localization_key(raw);
        let Some(text) = tally.accept(localization.get(key.as_ref())) else {
            tally.missing_labels += 1;
            continue;
        };
        tally.check_references(text);
        if language == GameLanguage::Russian && field == LabelField::Culture {
            tally.check_stem(text);
        }
        if text != english {
            labels.push(PackLabel {
                key: symbol,
                value: strings.intern(text)?,
            });
        }
    }
    let pack = StoredLanguagePack {
        format_version: PACK_FORMAT_VERSION,
        app_id: EU5_APP_ID,
        build_id: dataset.stored.build_id,
        location_count: count(dataset.stored.locations.len(), "location")?,
        symbol_count: count(dataset.stored.dictionary.len(), "symbol")?,
        dictionary_digest: dictionary_digest(&dataset.stored.dictionary),
        language,
        strings: strings.into_values(),
        location_names,
        labels,
        diagnostics: tally.into_diagnostics(),
        section: super::StoredLanguageSection::default(),
    };
    Ok(with_section(dataset, pack))
}

/// Reads one language from the install and builds its pack for `dataset`.
pub fn import_pack(
    installation: &GameInstallation,
    dataset: &Dataset,
    language: GameLanguage,
) -> Result<StoredLanguagePack, AppError> {
    if language.is_embedded() {
        return Err(AppError::InvalidData(
            "English text lives in the main bundles and has no pack".to_owned(),
        ));
    }
    let roots = installation.localization_roots(language);
    if roots.is_empty() {
        return Err(AppError::MissingFile(
            installation
                .root
                .join("game/main_menu/localization")
                .join(language.directory()),
        ));
    }
    let requested = requested_keys(dataset)?;
    let localization = read_localizations(&roots, &requested)?;
    build_pack(dataset, language, &localization)
}

/// Recomputes the section of an existing pack from its own strings, without the install.
///
/// Used by `reindex` after a section schema change; the pack must already pair with
/// `dataset`, otherwise this fails with `AppError::PackMismatch`.
pub fn rebuild_pack(
    dataset: &Dataset,
    pack: StoredLanguagePack,
) -> Result<StoredLanguagePack, AppError> {
    let paired = !pack.language.is_embedded()
        && pack.build_id == dataset.stored.build_id
        && usize::try_from(pack.location_count).ok() == Some(dataset.stored.locations.len())
        && usize::try_from(pack.symbol_count).ok() == Some(dataset.stored.dictionary.len())
        && pack.dictionary_digest == dictionary_digest(&dataset.stored.dictionary);
    if !paired {
        return Err(AppError::PackMismatch(format!(
            "{} pack for build {} cannot be rebuilt for build {}",
            pack.language.code(),
            pack.build_id,
            dataset.stored.build_id
        )));
    }
    let mut pack = pack;
    pack.format_version = PACK_FORMAT_VERSION;
    Ok(with_section(dataset, pack))
}

/// Every symbol shown through a label, in symbol order, with the field whose rule names its
/// localization key. Symbols used only as location keys are absent; their text is the name.
fn labelled_symbols(dataset: &Dataset) -> Result<BTreeMap<SymbolId, LabelField>, AppError> {
    let mut uses = BTreeMap::new();
    for record in &dataset.stored.locations {
        for field in LabelField::ALL {
            if let Some(symbol) = field.symbol(record) {
                let name = identifier(dataset, symbol)?;
                record_use(&mut uses, symbol, name, field)?;
            }
        }
    }
    Ok(uses)
}

/// Fills `section` from the pack's own resolved text.
fn with_section(dataset: &Dataset, mut pack: StoredLanguagePack) -> StoredLanguagePack {
    // The resolved text borrows the pack, so the section is computed before the assignment.
    let section = {
        let text = resolve_text(dataset, Some(&pack));
        crate::filter::build_language_section(dataset, &text)
    };
    pack.section = section;
    pack
}

fn identifier(dataset: &Dataset, symbol: SymbolId) -> Result<&str, AppError> {
    dataset.symbol(symbol).ok_or_else(|| {
        AppError::InvalidData(format!("symbol {} is outside the dictionary", symbol.0))
    })
}

fn count(value: usize, label: &str) -> Result<u32, AppError> {
    u32::try_from(value)
        .map_err(|error| AppError::InvalidData(format!("{label} count overflow: {error}")))
}
