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
