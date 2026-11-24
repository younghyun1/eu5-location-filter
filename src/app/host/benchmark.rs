//! Bounded real-data filter scans for browser optimization comparisons.

use std::sync::Arc;

use crate::filter::{FilterEngine, FilterSet, SortField};
use crate::{AppError, embedded};

/// Runs `iterations` filter passes over the embedded data and returns a result checksum.
pub(crate) fn benchmark_filter_work(iterations: u32) -> Result<u32, AppError> {
    if iterations > 10_000 {
        return Err(AppError::InvalidData(
            "browser benchmark iteration count exceeds 10,000".to_owned(),
        ));
    }
    let (dataset, index) = embedded::load()?;
    let engine = FilterEngine::from_stored_index(Arc::new(dataset), index)?;
    let queries = ["a", "river", "kourim", "newyork", "saint"];
    let mut checksum = 0_u32;
    for iteration in 0..iterations {
        let mut filters = FilterSet::land_only();
        let query = usize::try_from(iteration)
            .ok()
            .and_then(|value| queries.get(value % queries.len()))
            .copied()
            .unwrap_or_default();
        query.clone_into(&mut filters.search);
        let ids = engine.apply(&filters, SortField::Name, true);
        checksum = checksum.wrapping_add(u32::try_from(ids.len()).unwrap_or(u32::MAX));
    }
    Ok(checksum)
}
