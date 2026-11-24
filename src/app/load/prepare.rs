//! Turns decoded bundles into the dataset and engine the application installs.

use std::sync::Arc;

use crate::AppError;
use crate::filter::{FilterEngine, StoredFilterIndex};
use crate::model::Dataset;

/// A dataset and its engine, ready for `App::install_data`.
pub(crate) struct LoadedData {
    /// Shared dataset.
    pub(crate) dataset: Arc<Dataset>,
    /// Engine over the same dataset.
    pub(crate) engine: FilterEngine,
}

/// Restores the stored index, or builds one when an external bundle has none.
pub(crate) fn prepare(
    dataset: Dataset,
    index: Option<StoredFilterIndex>,
) -> Result<LoadedData, AppError> {
    let dataset = Arc::new(dataset);
    let engine = match index {
        Some(index) => FilterEngine::from_stored_index(Arc::clone(&dataset), index)?,
        None => FilterEngine::new(Arc::clone(&dataset)),
    };
    Ok(LoadedData { dataset, engine })
}

#[cfg(test)]
mod tests {
    use super::prepare;
    use crate::AppError;
    use crate::filter::test_fixture::fixture;
    use crate::filter::{FilterEngine, FilterSet, SortField};

    #[test]
    fn stored_and_rebuilt_indexes_give_the_same_results() -> Result<(), AppError> {
        let stored = FilterEngine::build_stored_index(&fixture());
        let restored = prepare(fixture(), Some(stored))?;
        let rebuilt = prepare(fixture(), None)?;
        let filters = FilterSet::land_only();
        assert_eq!(
            restored.engine.apply(&filters, SortField::Name, true),
            rebuilt.engine.apply(&filters, SortField::Name, true)
        );
        Ok(())
    }

    #[test]
    fn an_index_built_for_another_dataset_is_rejected() {
        let mut other = fixture();
        other.stored.build_id = other.stored.build_id.wrapping_add(1);
        let stored = FilterEngine::build_stored_index(&other);
        assert!(prepare(fixture(), Some(stored)).is_err());
    }
}
