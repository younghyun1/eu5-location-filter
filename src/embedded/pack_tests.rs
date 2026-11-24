//! Embedded language packs: each one that this binary carries must pair with the bundles.
//!
//! The build stages an empty file for a missing pack, so these tests skip absent languages;
//! a release build is expected to carry all ten.

use std::sync::Arc;

use super::{language_pack, load};
use crate::AppError;
use crate::filter::{FilterEngine, FilterSet, SortField};
use crate::lang_pack::{GameLanguage, decode_pack};
use crate::model::Dataset;

fn engine() -> Result<(Arc<Dataset>, FilterEngine), AppError> {
    let (dataset, index) = load()?;
    let dataset = Arc::new(dataset);
    let engine = FilterEngine::from_stored_index(Arc::clone(&dataset), index)?;
    Ok((dataset, engine))
}

fn keys_for(engine: &FilterEngine, dataset: &Dataset, search: &str) -> Vec<String> {
    let filters = FilterSet {
        search: search.to_owned(),
        ..FilterSet::default()
    };
    engine
        .apply(&filters, SortField::Name, true)
        .iter()
        .filter_map(|id| dataset.location(*id))
        .filter_map(|record| dataset.symbol(record.key))
        .map(str::to_owned)
        .collect()
}

#[test]
fn every_embedded_pack_decodes_validates_and_installs() -> Result<(), AppError> {
    assert_eq!(language_pack(GameLanguage::English), None);
    let (dataset, mut engine) = engine()?;
    let total = dataset.stored.locations.len();
    for language in GameLanguage::PACKED {
        let Some(bytes) = language_pack(language) else {
            eprintln!("{} pack is not embedded in this build", language.code());
            continue;
        };
        let pack = engine.validate_language_pack(decode_pack(bytes)?, language)?;
        engine.install_language_pack(pack)?;
        assert_eq!(engine.active_language(), language);
        for field in SortField::ALL {
            assert_eq!(
                engine.apply(&FilterSet::default(), field, true).len(),
                total
            );
        }
        assert_eq!(engine.drop_language_pack(), Some(language));
    }
    Ok(())
}

#[test]
fn korean_pack_finds_localized_and_english_names() -> Result<(), AppError> {
    let Some(bytes) = language_pack(GameLanguage::Korean) else {
        eprintln!("ko pack is not embedded in this build");
        return Ok(());
    };
    let (dataset, mut engine) = engine()?;
    let pack = engine.validate_language_pack(decode_pack(bytes)?, GameLanguage::Korean)?;
    engine.install_language_pack(pack)?;
    assert!(
        keys_for(&engine, &dataset, "스톡")
            .iter()
            .any(|key| key == "stockholm")
    );
    assert!(
        keys_for(&engine, &dataset, "stockholm")
            .iter()
            .any(|key| key == "stockholm")
    );
    assert_eq!(keys_for(&engine, &dataset, "N'Goussa"), ["ngoussa"]);
    Ok(())
}

#[test]
#[ignore = "offline deterministic regeneration of the committed language packs"]
fn committed_packs_rebuild_unchanged() -> Result<(), AppError> {
    let (dataset, _) = load()?;
    for language in GameLanguage::PACKED {
        let Some(bytes) = language_pack(language) else {
            continue;
        };
        let pack = decode_pack(bytes)?;
        let rebuilt = crate::lang_pack::rebuild_pack(&dataset, pack.clone())?;
        assert_eq!(rebuilt, pack, "{} pack changed on rebuild", language.code());
    }
    Ok(())
}
