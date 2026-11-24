//! Language pack files written by `import` and refreshed by `reindex`.
//!
//! Packs pair with one written location bundle (build ID, counts and dictionary digest), so
//! every function here takes the dataset as it was loaded back from disk.

use std::path::{Path, PathBuf};

use super::progress::{ImportProgress, ImportStage, report};
use crate::AppError;
use crate::lang_pack::{
    GameLanguage, ValidatedLanguagePack, import_pack, load_pack, pack_file_name, rebuild_pack,
    validate_pack, write_pack,
};
use crate::model::Dataset;
use crate::steam::GameInstallation;

/// Languages to process in schema order without repeats; an empty request means all eleven.
#[must_use]
pub fn selected_languages(requested: &[GameLanguage]) -> Vec<GameLanguage> {
    if requested.is_empty() {
        return GameLanguage::ALL.to_vec();
    }
    GameLanguage::ALL
        .into_iter()
        .filter(|language| requested.contains(language))
        .collect()
}

/// Location of a language's pack for one build inside `lang_dir`; `None` for English.
#[must_use]
pub fn pack_path(lang_dir: &Path, language: GameLanguage, build_id: u64) -> Option<PathBuf> {
    pack_file_name(language, build_id).map(|name| lang_dir.join(name))
}

/// Imports, validates and writes the pack of every non-English language in `languages`.
///
/// One failing language fails the whole call, because a release needs every pack it lists.
/// Returns the written paths in schema order.
pub fn write_language_packs(
    installation: &GameInstallation,
    dataset: &Dataset,
    languages: &[GameLanguage],
    lang_dir: &Path,
    force: bool,
    mut progress: impl FnMut(ImportProgress),
) -> Result<Vec<PathBuf>, AppError> {
    let packed: Vec<GameLanguage> = selected_languages(languages)
        .into_iter()
        .filter(|language| !language.is_embedded())
        .collect();
    let total = u64::try_from(packed.len()).unwrap_or(u64::MAX);
    let mut written = Vec::with_capacity(packed.len());
    for (done, language) in (0_u64..).zip(packed) {
        report(&mut progress, ImportStage::LanguagePacks, done, total);
        let Some(path) = pack_path(lang_dir, language, dataset.stored.build_id) else {
            continue;
        };
        let stored = import_pack(installation, dataset, language)?;
        let validated = validate_pack(stored, dataset, language)?;
        write_pack(&path, validated.stored(), force)?;
        written.push(path);
    }
    report(&mut progress, ImportStage::LanguagePacks, total, total);
    Ok(written)
}

/// Recomputes the section of every pack in `lang_dir` for this dataset's build and writes it
/// back in place; languages without a pack file are skipped.
///
/// Replacing the file is the purpose of a reindex, so packs are always overwritten. Returns
/// the rewritten paths in schema order.
pub fn reindex_language_packs(
    dataset: &Dataset,
    lang_dir: &Path,
) -> Result<Vec<PathBuf>, AppError> {
    let mut written = Vec::new();
    for language in GameLanguage::PACKED {
        let Some(path) = pack_path(lang_dir, language, dataset.stored.build_id) else {
            continue;
        };
        if !path.is_file() {
            continue;
        }
        let rebuilt = rebuild_pack(dataset, load_pack(&path)?)?;
        let validated = validate_pack(rebuilt, dataset, language)?;
        write_pack(&path, validated.stored(), true)?;
        written.push(path);
    }
    Ok(written)
}

/// Loads and validates the pack for `language` that pairs with `dataset`, or `None` when
/// English is requested or `lang_dir` holds no pack for this build.
///
/// A file that exists but fails to decode or pair is an error, so a stale pack is reported
/// instead of silently compared as English.
pub fn load_paired_pack(
    dataset: &Dataset,
    lang_dir: &Path,
    language: GameLanguage,
) -> Result<Option<ValidatedLanguagePack>, AppError> {
    let Some(path) = pack_path(lang_dir, language, dataset.stored.build_id) else {
        return Ok(None);
    };
    if !path.is_file() {
        return Ok(None);
    }
    let stored = load_pack(&path)?;
    validate_pack(stored, dataset, language).map(Some)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::{load_paired_pack, pack_path, reindex_language_packs, selected_languages};
    use crate::AppError;
    use crate::filter::test_fixture::{fixture, fixture_pack};
    use crate::lang_pack::{GameLanguage, load_pack, write_pack};

    #[test]
    fn selection_defaults_to_all_and_keeps_schema_order() {
        assert_eq!(selected_languages(&[]), GameLanguage::ALL.to_vec());
        let picked = selected_languages(&[
            GameLanguage::Korean,
            GameLanguage::French,
            GameLanguage::Korean,
        ]);
        assert_eq!(picked, vec![GameLanguage::French, GameLanguage::Korean]);
        assert_eq!(pack_path(Path::new("lang"), GameLanguage::English, 1), None);
        assert_eq!(
            pack_path(Path::new("lang"), GameLanguage::Korean, 7),
            Some(Path::new("lang").join("ko.7.bitcode.zst"))
        );
    }

    #[test]
    fn reindex_rewrites_existing_packs_and_diff_loading_pairs_them() -> Result<(), AppError> {
        let directory = std::env::temp_dir().join(format!(
            "eu5-location-filter-import-packs-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory)
            .map_err(|source| AppError::io("create", &directory, source))?;
        let dataset = fixture();
        let mut stale = fixture_pack(&dataset);
        let expected = stale.clone();
        stale.section.searchable.clear();
        let path = directory.join("ko.1.bitcode.zst");
        write_pack(&path, &stale, false)?;
        assert!(load_paired_pack(&dataset, &directory, GameLanguage::Korean).is_err());
        let written = reindex_language_packs(&dataset, &directory)?;
        assert_eq!(written, vec![path.clone()]);
        assert_eq!(load_pack(&path)?, expected);
        let paired = load_paired_pack(&dataset, &directory, GameLanguage::Korean)?;
        assert_eq!(
            paired.map(|pack| pack.language()),
            Some(GameLanguage::Korean)
        );
        let absent = load_paired_pack(&dataset, &directory, GameLanguage::French)?;
        assert!(absent.is_none());
        let _ = fs::remove_dir_all(directory);
        Ok(())
    }
}
