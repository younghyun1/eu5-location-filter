//! Build-time embedded location and filter-index bundles.

use crate::filter::StoredFilterIndex;
use crate::model::Dataset;
use crate::{AppError, index_storage, storage};

const LOCATIONS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/eu5-locations.bitcode.zst"));
const INDEXES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/eu5-indexes.bitcode.zst"));

/// Decodes both committed bundles into memory without accessing game files.
pub fn load() -> Result<(Dataset, StoredFilterIndex), AppError> {
    if LOCATIONS.is_empty() || INDEXES.is_empty() {
        return Err(AppError::InvalidData(
            "embedded EU5 bundles are missing from this build".to_owned(),
        ));
    }
    Ok((
        storage::decode_blob(LOCATIONS)?,
        index_storage::decode_index(INDEXES)?,
    ))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::load;
    use crate::filter::{FilterEngine, FilterSet, SortField};

    #[test]
    fn committed_bundles_decode_and_pair() {
        let loaded = load();
        assert!(loaded.is_ok());
        let Ok((dataset, index)) = loaded else {
            return;
        };
        assert_eq!(dataset.stored.locations.len(), 28_573);
        let engine = FilterEngine::from_stored_index(Arc::new(dataset), index);
        assert!(engine.is_ok());
        let Ok(engine) = engine else { return };
        assert_eq!(
            engine
                .apply(&FilterSet::default(), SortField::Name, true)
                .len(),
            28_573
        );
    }
}
