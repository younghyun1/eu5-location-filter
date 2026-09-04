//! Dependency-free, interleaved dev-profile comparisons against the previous scan.

use super::{FilterEngine, FilterSet, FloatRange, SortField};
use crate::{
    AppError,
    model::{LocationId, SymbolId},
};
use std::{
    hint::black_box,
    sync::Arc,
    time::{Duration, Instant},
};

#[test]
#[ignore = "local timing comparison over committed bundles"]
fn compares_indexed_queries_to_scan() -> Result<(), AppError> {
    let started = Instant::now();
    let (dataset, index) = crate::embedded::load()?;
    let engine = FilterEngine::from_stored_index(Arc::new(dataset), index)?;
    eprintln!("decode and validate: {:?}", started.elapsed());
    let mut facets = FilterSet::land_only();
    facets.coastal.insert(true);
    facets.food_producing_only = true;
    let mut cases = vec![
        ("all", FilterSet::default()),
        ("land", FilterSet::land_only()),
        ("facets", facets),
        (
            "numeric",
            FilterSet {
                harbor_range: FloatRange {
                    min: Some(0.5),
                    max: Some(0.75),
                },
                ..FilterSet::default()
            },
        ),
        (
            "short search",
            FilterSet {
                search: "a".to_owned(),
                ..FilterSet::default()
            },
        ),
        (
            "substring",
            FilterSet {
                search: "stock".to_owned(),
                ..FilterSet::default()
            },
        ),
        (
            "absent search",
            FilterSet {
                search: "zzzzzzzzzz".to_owned(),
                ..FilterSet::default()
            },
        ),
    ];
    if let Some(record) = engine.dataset.stored.locations.first() {
        cases.push((
            "province",
            FilterSet {
                provinces: [Some(record.hierarchy.province)].into_iter().collect(),
                ..FilterSet::default()
            },
        ));
        cases.push((
            "exact RGB",
            FilterSet {
                rgb: Some(record.color),
                ..FilterSet::default()
            },
        ));
    }
    let invalid = FilterSet {
        cultures: [Some(SymbolId(u32::MAX))].into_iter().collect(),
        ..FilterSet::default()
    };
    cases.push(("absent facet", invalid));
    for (name, filters) in &cases {
        for field in SortField::ALL {
            for ascending in [true, false] {
                assert_eq!(
                    engine.apply(filters, field, ascending),
                    engine.apply_scan(filters, field, ascending)
                );
            }
        }
        measure(
            name,
            || engine.apply(filters, SortField::Name, true),
            || engine.apply_scan(filters, SortField::Name, true),
        );
    }
    Ok(())
}

fn measure(name: &str, indexed: impl Fn() -> Vec<LocationId>, scan: impl Fn() -> Vec<LocationId>) {
    for _ in 0..10 {
        black_box(indexed());
        black_box(scan());
    }
    let mut fast = Vec::with_capacity(101);
    let mut slow = Vec::with_capacity(101);
    for iteration in 0..101 {
        // Alternate order to reduce systematic thermal and cache bias.
        if iteration % 2 == 0 {
            fast.push(elapsed(&indexed));
            slow.push(elapsed(&scan));
        } else {
            slow.push(elapsed(&scan));
            fast.push(elapsed(&indexed));
        }
    }
    fast.sort_unstable();
    slow.sort_unstable();
    if let (Some(fast_median), Some(slow_median), Some(fast_p95), Some(slow_p95)) =
        (fast.get(50), slow.get(50), fast.get(95), slow.get(95))
    {
        eprintln!(
            "{name}: rows={}; indexed median={fast_median:?}, p95={fast_p95:?}; scan median={slow_median:?}, p95={slow_p95:?}; speedup={:.2}x",
            indexed().len(),
            slow_median.as_secs_f64() / fast_median.as_secs_f64()
        );
    }
}

fn elapsed(operation: &impl Fn() -> Vec<LocationId>) -> Duration {
    let started = Instant::now();
    black_box(operation());
    started.elapsed()
}

#[test]
#[ignore = "local output-order crossover comparison over committed bundles"]
fn compares_sparse_sort_strategy() -> Result<(), AppError> {
    let (dataset, index) = crate::embedded::load()?;
    let engine = FilterEngine::from_stored_index(Arc::new(dataset), index)?;
    let total = engine.dataset.stored.locations.len();
    assert!(total > 0);
    let Some(order) = engine.sort_orders.get(SortField::Name, true) else {
        return Err(AppError::InvalidData(
            "name sort order is missing".to_owned(),
        ));
    };
    for count in [1, 16, 64, 256, 512, 1024, 4096, 16384, total] {
        let mut mask = super::bitmap::Bitmap::empty(total);
        // Spread IDs across the dataset instead of accidentally benchmarking already
        // ordered contiguous records. This stride is coprime to the bundled row count.
        for value in 0..count {
            if let Ok(id) = u32::try_from(value * 7919 % total) {
                mask.insert(LocationId(id));
            }
        }
        let indexed = || engine.sort_orders.select(&mask, SortField::Name, true);
        let scanned = || {
            order
                .iter()
                .copied()
                .filter(|id| mask.contains(*id))
                .collect::<Vec<_>>()
        };
        assert_eq!(indexed(), scanned());
        measure(
            &format!(
                "sort {count}, ranks={}",
                super::sort::use_ranks(count, total)
            ),
            indexed,
            scanned,
        );
    }
    Ok(())
}

#[test]
#[ignore = "offline deterministic regeneration of the committed index"]
fn verifies_committed_index_regeneration() -> Result<(), AppError> {
    let (dataset, stored) = crate::embedded::load()?;
    let rebuilt = FilterEngine::build_stored_index(&dataset);
    rebuilt.validate(&dataset)?;
    assert_eq!(stored, rebuilt);
    let encoded = crate::index_storage::encode_index(&rebuilt)?;
    assert_eq!(
        encoded.as_slice(),
        include_bytes!("../../assets/eu5-indexes.bitcode.zst")
    );
    assert_eq!(
        encoded,
        crate::index_storage::encode_index(&FilterEngine::build_stored_index(&dataset))?
    );
    eprintln!(
        "index payload: {} bytes; zstd22 bundle: {} bytes",
        bitcode::encode(&rebuilt).len(),
        encoded.len()
    );
    Ok(())
}
