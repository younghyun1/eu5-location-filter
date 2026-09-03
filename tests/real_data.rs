//! Ignored checks over a locally generated blob. No game data is packaged with the tests.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use eu5_location_filter::filter::{FilterEngine, FilterSet, SortField};
use eu5_location_filter::model::{LocationKind, MAX_RIVER_LEVEL};
use eu5_location_filter::storage;

#[test]
#[ignore = "requires a local build-24187685 data blob"]
fn verifies_reference_install_import() -> Result<(), eu5_location_filter::AppError> {
    let dataset = load_local_blob()?;
    assert_eq!(dataset.stored.build_id, 24_187_685);
    assert_eq!(dataset.stored.locations.len(), 28_573);
    assert_eq!(
        dataset
            .stored
            .locations
            .iter()
            .filter(|record| record.kind == LocationKind::Impassable)
            .count(),
        1_870
    );
    assert!(dataset.stored.locations.iter().any(|record| {
        dataset.symbol(record.key) == Some("heard_island")
            && record.kind == LocationKind::Impassable
            && record.static_population_capacity.is_none()
    }));
    assert!(dataset.stored.locations.iter().all(|record| {
        record
            .river
            .is_none_or(|river| (1..=MAX_RIVER_LEVEL).contains(&river.level.0))
    }));
    let river_locations: Vec<_> = dataset
        .stored
        .locations
        .iter()
        .filter_map(|record| record.river)
        .collect();
    assert!(river_locations.len() > 1_000);
    assert!(river_locations.iter().any(|river| river.has_source));
    assert!(river_locations.iter().any(|river| river.has_confluence));
    assert_eq!(
        river_locations.iter().map(|river| river.level.0).max(),
        Some(MAX_RIVER_LEVEL)
    );
    assert!(
        dataset
            .stored
            .locations
            .iter()
            .filter(|record| record.kind == LocationKind::Land && record.vegetation.is_some())
            .all(|record| record.static_population_capacity.is_some())
    );
    assert_eq!(dataset.by_color.len(), dataset.stored.locations.len());
    assert_eq!(dataset.by_key.len(), dataset.stored.locations.len());
    assert!(dataset.stored.locations.iter().all(|record| {
        dataset
            .symbol(record.name)
            .is_some_and(|name| !name.contains('$'))
    }));
    Ok(())
}

#[test]
#[ignore = "timing harness over the committed embedded bundles"]
fn measures_repeated_filter_latency() -> Result<(), eu5_location_filter::AppError> {
    let load_started = Instant::now();
    let (dataset, index) = eu5_location_filter::embedded::load()?;
    let engine = FilterEngine::from_stored_index(Arc::new(dataset), index)?;
    let load_time = load_started.elapsed();
    let full = benchmark(&engine, &FilterSet::default(), 100);
    let search = benchmark(
        &engine,
        &FilterSet {
            search: "stock".to_owned(),
            ..FilterSet::default()
        },
        100,
    );
    let mut sort_max = Duration::ZERO;
    for field in SortField::ALL {
        let started = Instant::now();
        std::hint::black_box(engine.apply(&FilterSet::default(), field, false));
        sort_max = sort_max.max(started.elapsed());
    }
    eprintln!(
        "bundle load: {load_time:?}; full-filter max: {full:?}; name-search max: {search:?}; sort-scan max: {sort_max:?}"
    );
    assert!(full < Duration::from_millis(17));
    assert!(search < Duration::from_millis(17));
    assert!(sort_max < Duration::from_millis(17));
    Ok(())
}

fn benchmark(engine: &FilterEngine, filters: &FilterSet, iterations: usize) -> Duration {
    let mut maximum = Duration::ZERO;
    for _ in 0..iterations {
        let started = Instant::now();
        let result = engine.apply(filters, SortField::Name, true);
        std::hint::black_box(result);
        maximum = maximum.max(started.elapsed());
    }
    maximum
}

fn load_local_blob() -> Result<eu5_location_filter::model::Dataset, eu5_location_filter::AppError> {
    let path = std::env::var_os("EU5_DATA_FILE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("eu5-locations.bitcode.zst"));
    storage::load_dataset(&path)
}
