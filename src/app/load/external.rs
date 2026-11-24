//! Desktop bundle sources: the embedded bundles, external bundle files with their index, and
//! the import from the game directory that rewrites the external files.

use std::path::{Path, PathBuf};

use super::progress::{fraction, import_stage_text};
use crate::app::i18n::Text;
use crate::app::state::LaunchPaths;
use crate::filter::{FilterEngine, StoredFilterIndex};
use crate::import::import_game;
use crate::model::Dataset;
use crate::{AppError, embedded, index_storage, steam, storage};

/// Data file written by a rebuild when the command line named none.
const DEFAULT_DATA_FILE: &str = "eu5-locations.bitcode.zst";

/// Loads the dataset and its index, or imports and writes them when `force` is set or the named
/// data file does not exist. `report` receives each stage and its completed fraction.
///
/// The index is `None` only for an external data file that has no index beside it; the caller
/// then builds one in memory.
pub(super) fn load_or_import(
    launch: &LaunchPaths,
    force: bool,
    report: &mut dyn FnMut(Text, f32),
) -> Result<(Dataset, Option<StoredFilterIndex>), AppError> {
    let data_file = launch.data_file.as_deref();
    if !force && data_file.is_none() {
        report(Text::LoadEmbedded, 0.0);
        return embedded::load().map(|(dataset, index)| (dataset, Some(index)));
    }
    if !force && let Some(data_path) = data_file.filter(|path| path.is_file()) {
        report(Text::LoadLoadingData, 0.0);
        let dataset = storage::load_dataset(data_path)?;
        let index_path = launch
            .index_file
            .clone()
            .unwrap_or_else(|| inferred_index_path(data_path));
        let index = if index_path.is_file() {
            Some(index_storage::load_index(&index_path)?)
        } else {
            None
        };
        return Ok((dataset, index));
    }
    import_and_write(launch, force, report)
}

/// Imports from the installation, writes both external bundles, and returns them as loaded
/// back from disk so a write that cannot be read again fails here.
fn import_and_write(
    launch: &LaunchPaths,
    force: bool,
    report: &mut dyn FnMut(Text, f32),
) -> Result<(Dataset, Option<StoredFilterIndex>), AppError> {
    report(Text::LoadRebuild, 0.0);
    let data_path = launch
        .data_file
        .as_deref()
        .map_or_else(|| PathBuf::from(DEFAULT_DATA_FILE), Path::to_path_buf);
    let index_path = launch
        .index_file
        .clone()
        .unwrap_or_else(|| inferred_index_path(&data_path));
    let installation = steam::discover(launch.game_dir.as_deref())?;
    let stored = import_game(&installation, |progress| {
        report(
            import_stage_text(progress.stage),
            fraction(progress.current, progress.total),
        );
    })?;
    storage::write_dataset(&data_path, &stored, force)?;
    let dataset = storage::load_dataset(&data_path)?;
    report(Text::LoadExternal, 0.0);
    let index = FilterEngine::build_stored_index(&dataset);
    index.validate(&dataset)?;
    index_storage::write_index(&index_path, &index, force)?;
    Ok((dataset, Some(index)))
}

/// Index file beside `data_path`: `<stem>.indexes.bitcode.zst`.
fn inferred_index_path(data_path: &Path) -> PathBuf {
    let mut name = data_path
        .file_stem()
        .map_or_else(|| "eu5-locations".into(), |value| value.to_os_string());
    name.push(".indexes.bitcode.zst");
    data_path.with_file_name(name)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{inferred_index_path, load_or_import};
    use crate::AppError;
    use crate::app::i18n::Text;
    use crate::app::state::LaunchPaths;

    fn launch(data_file: Option<&str>, game_dir: Option<&str>) -> LaunchPaths {
        LaunchPaths {
            data_file: data_file.map(PathBuf::from),
            index_file: None,
            game_dir: game_dir.map(PathBuf::from),
        }
    }

    #[test]
    fn index_path_follows_the_data_file_stem() {
        assert_eq!(
            inferred_index_path(Path::new("dir/eu5-locations.bitcode.zst")),
            Path::new("dir/eu5-locations.bitcode.indexes.bitcode.zst")
        );
        assert_eq!(
            inferred_index_path(Path::new("")),
            Path::new("eu5-locations.indexes.bitcode.zst")
        );
    }

    #[test]
    fn embedded_load_reports_its_stage_first() -> Result<(), AppError> {
        let mut stages = Vec::new();
        let (dataset, index) = load_or_import(&launch(None, None), false, &mut |stage, _| {
            stages.push(stage);
        })?;
        assert_eq!(stages, vec![Text::LoadEmbedded]);
        assert!(index.is_some());
        assert!(!dataset.stored.locations.is_empty());
        Ok(())
    }

    #[test]
    fn rebuild_with_a_missing_installation_fails_after_announcing_itself() {
        let mut stages = Vec::new();
        let result = load_or_import(
            &launch(Some("/nonexistent/eu5-test.bitcode.zst"), Some("/nonexistent/eu5-game")),
            true,
            &mut |stage, _| stages.push(stage),
        );
        assert!(result.is_err());
        assert_eq!(stages, vec![Text::LoadRebuild]);
    }

    #[test]
    fn a_missing_data_file_falls_back_to_import() {
        let mut stages = Vec::new();
        let result = load_or_import(
            &launch(Some("/nonexistent/eu5-test.bitcode.zst"), Some("/nonexistent/eu5-game")),
            false,
            &mut |stage, _| stages.push(stage),
        );
        assert!(result.is_err());
        assert_eq!(stages, vec![Text::LoadRebuild]);
    }
}
