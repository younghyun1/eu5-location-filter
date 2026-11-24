//! Offline pack construction from an EU5 install and a validated dataset.

use std::collections::{HashMap, HashSet};

use super::codec::not_implemented;
use super::{GameLanguage, StoredLanguagePack};
use crate::AppError;
use crate::model::Dataset;
use crate::steam::GameInstallation;

/// Localization keys a pack needs: every location key plus each labelled field's key.
///
/// Fails when one symbol is used both as a static modifier and in another labelled field,
/// because the two fields resolve labels through different localization keys.
pub fn requested_keys(dataset: &Dataset) -> Result<HashSet<String>, AppError> {
    let _ = dataset;
    Err(not_implemented("key collection"))
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
    let _ = (dataset, language, localization);
    Err(not_implemented("building"))
}

/// Reads one language from the install and builds its pack for `dataset`.
pub fn import_pack(
    installation: &GameInstallation,
    dataset: &Dataset,
    language: GameLanguage,
) -> Result<StoredLanguagePack, AppError> {
    let _ = (installation, dataset, language);
    Err(not_implemented("import"))
}

/// Recomputes the section of an existing pack from its own strings, without the install.
///
/// Used by `reindex` after a section schema change; the pack must already pair with
/// `dataset`.
pub fn rebuild_pack(
    dataset: &Dataset,
    pack: StoredLanguagePack,
) -> Result<StoredLanguagePack, AppError> {
    let _ = (dataset, pack);
    Err(not_implemented("rebuilding"))
}
