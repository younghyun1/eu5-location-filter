//! Engine behavior with an installed language pack: search, labels, orders, and pairing.

use std::collections::HashSet;
use std::sync::Arc;

use super::indexed_tests::{equal_to_scan, varied_fixture};
use super::test_fixture::{add_english_label, fixture, fixture_pack, pack_with};
use super::text::fold_sort;
use super::{FilterEngine, FilterSet, SortField};
use crate::AppError;
use crate::lang_pack::{GameLanguage, LabelField, validate_pack};
use crate::model::{Dataset, LocationId, LocationKind, SymbolId};

fn search(text: &str) -> FilterSet {
    FilterSet {
        search: text.to_owned(),
        ..FilterSet::default()
    }
}

/// Korean names for the three name symbols of the varied fixture (one keeps English) and
/// labels that reorder several category symbols relative to their identifiers.
fn varied_pack_engine() -> Result<FilterEngine, AppError> {
    let dataset = varied_fixture();
    let names: Vec<Option<&str>> = dataset
        .stored
        .locations
        .iter()
        .map(|record| match record.name.0 {
            7 => Some("도쿄 아아"),
            8 => Some("라스 알 아인"),
            _ => None,
        })
        .collect();
    let labels = [
        (SymbolId(0), "하늘"),
        (SymbolId(2), "평지"),
        (SymbolId(3), "가톨릭"),
        (SymbolId(5), "밀"),
    ];
    let pack = pack_with(&dataset, GameLanguage::Korean, &names, &labels);
    let mut engine = FilterEngine::new(Arc::new(dataset));
    let validated = engine.validate_language_pack(pack, GameLanguage::Korean)?;
    engine.install_language_pack(validated)?;
    Ok(engine)
}

#[test]
fn installed_pack_searches_and_labels_then_drop_restores_english() -> Result<(), AppError> {
    let dataset = Arc::new(fixture());
    let mut engine = FilterEngine::new(Arc::clone(&dataset));
    let english = engine.apply(&FilterSet::default(), SortField::Name, true);
    assert!(
        engine
            .apply(&search("에이"), SortField::Name, true)
            .is_empty()
    );
    let pack = engine.validate_language_pack(fixture_pack(&dataset), GameLanguage::Korean)?;
    engine.install_language_pack(pack)?;
    assert_eq!(engine.active_language(), GameLanguage::Korean);
    assert_eq!(
        engine.apply(&search("에이"), SortField::Name, true),
        vec![LocationId(0)]
    );
    // English names and identifiers stay searchable while another language is active.
    assert_eq!(
        engine.apply(&search("Éire"), SortField::Name, true),
        vec![LocationId(0)]
    );
    assert_eq!(
        engine.apply(&search("waste"), SortField::Name, true),
        vec![LocationId(1)]
    );
    let labels = engine.labels();
    assert_eq!(labels.language(), GameLanguage::Korean);
    assert_eq!(labels.name(LocationId(0)), Some("에이레"));
    assert_eq!(labels.label(SymbolId(2)), Some("평지"));
    assert_eq!(labels.label(SymbolId(3)), Some("catholic"));
    assert_eq!(engine.drop_language_pack(), Some(GameLanguage::Korean));
    assert_eq!(engine.drop_language_pack(), None);
    assert_eq!(engine.active_language(), GameLanguage::English);
    assert_eq!(
        engine.apply(&FilterSet::default(), SortField::Name, true),
        english
    );
    assert!(
        engine
            .apply(&search("에이"), SortField::Name, true)
            .is_empty()
    );
    assert_eq!(engine.labels().name(LocationId(0)), Some("Éire"));
    Ok(())
}

#[test]
fn search_text_keeps_the_english_form_and_prefixes_localized_names() -> Result<(), AppError> {
    let dataset = fixture();
    let index = FilterEngine::build_stored_index(&dataset);
    assert_eq!(index.english.searchable, ["eire\0eire", "waste\0waste"]);
    let pack = fixture_pack(&dataset);
    assert_eq!(
        pack.section.searchable,
        ["에이레\0eire\0eire", "황무지\0waste\0waste"]
    );
    validate_pack(pack, &dataset, GameLanguage::Korean).map(|_| ())
}

#[test]
fn pack_validated_for_another_dataset_cannot_install() -> Result<(), AppError> {
    let original = fixture();
    let mut changed = fixture();
    changed.stored.dictionary[3] = "orthodox".to_owned();
    let pack = validate_pack(fixture_pack(&original), &original, GameLanguage::Korean)?;
    let mut engine = FilterEngine::new(Arc::new(changed));
    let installed = engine.install_language_pack(pack);
    assert!(matches!(installed, Err(AppError::PackMismatch(_))));
    assert_eq!(engine.active_language(), GameLanguage::English);
    let revalidated = engine.validate_language_pack(fixture_pack(&original), GameLanguage::Korean);
    assert!(matches!(revalidated, Err(AppError::PackMismatch(_))));
    Ok(())
}

#[test]
fn engine_and_free_validation_agree() -> Result<(), AppError> {
    let dataset = Arc::new(fixture());
    let engine = FilterEngine::new(Arc::clone(&dataset));
    let by_engine = engine.validate_language_pack(fixture_pack(&dataset), GameLanguage::Korean)?;
    let free = validate_pack(fixture_pack(&dataset), &dataset, GameLanguage::Korean)?;
    assert_eq!(by_engine.stored(), free.stored());
    assert!(Arc::ptr_eq(engine.dataset(), &dataset));
    Ok(())
}

#[test]
fn symbol_columns_sort_by_label_not_identifier() {
    let mut dataset = fixture();
    dataset.stored.locations[1].topography = SymbolId(3);
    let by_identifier = FilterEngine::new(Arc::new(dataset.clone()));
    // "catholic" sorts before "flatland" while both labels fall back to their identifiers.
    assert_eq!(
        by_identifier.apply(&FilterSet::default(), SortField::Topography, true),
        vec![LocationId(1), LocationId(0)]
    );
    add_english_label(&mut dataset, SymbolId(3), "Zz plains");
    let by_label = FilterEngine::new(Arc::new(dataset));
    assert_eq!(
        by_label.apply(&FilterSet::default(), SortField::Topography, true),
        vec![LocationId(0), LocationId(1)]
    );
    assert_eq!(
        by_label.apply(&FilterSet::default(), SortField::Topography, false),
        vec![LocationId(1), LocationId(0)]
    );
}

#[test]
fn name_order_follows_the_installed_pack() -> Result<(), AppError> {
    let dataset = Arc::new(fixture());
    let mut engine = FilterEngine::new(Arc::clone(&dataset));
    assert_eq!(
        engine.apply(&FilterSet::default(), SortField::Name, true),
        vec![LocationId(0), LocationId(1)]
    );
    let pack = pack_with(
        &dataset,
        GameLanguage::Korean,
        &[Some("하"), Some("가")],
        &[],
    );
    let validated = engine.validate_language_pack(pack, GameLanguage::Korean)?;
    engine.install_language_pack(validated)?;
    assert_eq!(
        engine.apply(&FilterSet::default(), SortField::Name, true),
        vec![LocationId(1), LocationId(0)]
    );
    // Identifier order never depends on the language.
    assert_eq!(
        engine.apply(&FilterSet::default(), SortField::Identifier, true),
        vec![LocationId(0), LocationId(1)]
    );
    Ok(())
}

#[test]
fn installed_pack_matches_scan_for_every_column() -> Result<(), AppError> {
    let engine = varied_pack_engine()?;
    equal_to_scan(&engine, &FilterSet::default());
    equal_to_scan(&engine, &FilterSet::land_only());
    for query in [
        "도쿄",
        "쿄",
        "도쿄아아",
        "라스",
        "아인",
        "알아",
        "ras",
        "kourim",
        "東京",
        "eire",
        "아a",
        "zzzz",
        "",
    ] {
        equal_to_scan(&engine, &search(query));
    }
    let choices = HashSet::from([None, Some(SymbolId(0)), Some(SymbolId(5))]);
    let mut filters = FilterSet {
        religions: choices.clone(),
        cultures: choices,
        search: "아".to_owned(),
        ..FilterSet::default()
    };
    filters
        .kinds
        .extend([LocationKind::Land, LocationKind::Sea]);
    equal_to_scan(&engine, &filters);
    Ok(())
}

#[test]
fn section_orders_follow_resolved_labels_with_nulls_last() -> Result<(), AppError> {
    let engine = varied_pack_engine()?;
    assert_label_order(&engine);
    let english = FilterEngine::new(Arc::new(varied_fixture()));
    assert_label_order(&english);
    Ok(())
}

/// Independent oracle: every label-ordered column is non-decreasing by folded display text
/// in ascending order, with nulls after every value.
fn assert_label_order(engine: &FilterEngine) {
    let labels = engine.labels();
    let dataset: &Dataset = labels.dataset();
    let all = FilterSet::default();
    let names: Vec<String> = engine
        .apply(&all, SortField::Name, true)
        .iter()
        .map(|id| fold_sort(labels.name(*id).unwrap_or_default()))
        .collect();
    assert!(
        names
            .windows(2)
            .all(|pair| matches!(pair, [left, right] if left <= right))
    );
    for field in LabelField::ALL {
        let keys: Vec<Option<String>> = engine
            .apply(&all, field.sort_field(), true)
            .iter()
            .map(|id| {
                dataset
                    .location(*id)
                    .and_then(|record| field.symbol(record))
                    .and_then(|symbol| labels.label(symbol))
                    .map(fold_sort)
            })
            .collect();
        let ordered = keys.windows(2).all(|pair| match pair {
            [Some(left), Some(right)] => left <= right,
            [None, Some(_)] => false,
            _ => true,
        });
        assert!(ordered, "{field:?} is not ordered by its label");
    }
}
