//! Option list building, key lookup and folded option search.

use super::options::test_data::{
    HILLS, KOURIM, WHEAT, dataset, english_options, korean_pack,
};
use super::{FacetId, FacetOption, FacetOptions, OptionList, OptionValue};
use crate::app::i18n::Locale;
use crate::lang_pack::Labels;
use crate::model::{LocationKind, SymbolId};

fn labels_of(list: &OptionList) -> Vec<&str> {
    (0..list.len())
        .filter_map(|index| u32::try_from(index).ok())
        .map(|index| list.label(index))
        .collect()
}

fn keys_of(list: &OptionList) -> Vec<&str> {
    (0..list.len())
        .filter_map(|index| u32::try_from(index).ok())
        .filter_map(|index| list.get(index))
        .map(|option| option.key.as_str())
        .collect()
}

fn search(list: &OptionList, query: &str) -> Vec<u32> {
    let mut out = vec![99];
    list.filter_into(query, &mut out);
    out
}

fn free_list(labels: &[&str]) -> OptionList {
    OptionList::new(
        labels
            .iter()
            .enumerate()
            .map(|(index, label)| {
                FacetOption::new(
                    OptionValue::Symbol(Some(SymbolId(u32::try_from(index).unwrap_or(0)))),
                    &format!("key{index}"),
                    (*label).to_owned(),
                )
            })
            .collect(),
    )
}

#[test]
fn static_lists_keep_contract_order_and_wire_keys() {
    let data = dataset();
    let options = english_options(&data);
    let kinds = options.get(FacetId::Kind);
    assert_eq!(keys_of(kinds), ["land", "sea", "lake", "impassable", "unknown"]);
    assert_eq!(labels_of(kinds), ["Land", "Sea", "Lake", "Impassable", "Unknown"]);
    assert_eq!(kinds.value(1), Some(OptionValue::Kind(LocationKind::Sea)));
    let coastal = options.get(FacetId::Coastal);
    assert_eq!(keys_of(coastal), ["yes", "no"]);
    assert_eq!(labels_of(coastal), ["Yes", "No"]);
    assert_eq!(coastal.value(0), Some(OptionValue::Flag(true)));
    assert_eq!(coastal.value(1), Some(OptionValue::Flag(false)));
    for facet in [FacetId::RiverPresence, FacetId::HarborPresence, FacetId::MovementPresence] {
        let list = options.get(facet);
        assert_eq!(keys_of(list), ["present", "missing"], "{facet:?}");
        assert_eq!(labels_of(list), ["Present", "Missing"], "{facet:?}");
        assert_eq!(list.value(0), Some(OptionValue::Flag(true)));
    }
}

#[test]
fn river_tiers_are_shared_between_the_bound_facets() {
    let data = dataset();
    let options = english_options(&data);
    let min = options.get(FacetId::RiverLevelMin);
    let max = options.get(FacetId::RiverLevelMax);
    assert!(std::ptr::eq(min, max));
    assert_eq!(keys_of(min), ["1", "2", "3", "4", "5"]);
    assert_eq!(
        labels_of(min),
        [
            "Level 1: Brook (+10%)",
            "Level 2: Stream (+20%)",
            "Level 3: River (+30%)",
            "Level 4: Large river (+40%)",
            "Level 5: Major river (+50%)",
        ]
    );
    assert_eq!(min.find_key("3"), Some(OptionValue::Tier(3)));
    assert_eq!(min.find_key("6"), None);
}

#[test]
fn dataset_lists_are_sorted_by_folded_label_and_show_keys() {
    let data = dataset();
    let options = english_options(&data);
    let continents = options.get(FacetId::Continent);
    assert_eq!(labels_of(continents), ["Europe [europe]"]);
    assert_eq!(keys_of(continents), ["europe"]);
    assert_eq!(continents.value(0), Some(OptionValue::Symbol(Some(SymbolId(7)))));
    assert_eq!(labels_of(options.get(FacetId::Subcontinent)), ["Kouřim [kourim]"]);
    assert_eq!(labels_of(options.get(FacetId::Region)), ["Ra's al-'Ain [ras_al_ain]"]);
    assert_eq!(labels_of(options.get(FacetId::Area)), ["Москва [moskva]"]);
    assert_eq!(labels_of(options.get(FacetId::Province)), ["東京 [tokyo]"]);
    // A label equal to its key shows the key once.
    assert_eq!(labels_of(options.get(FacetId::Topography)), ["hills", "mountains", "ocean"]);
    assert_eq!(options.get(FacetId::Topography).value(0), Some(OptionValue::Symbol(Some(HILLS))));
}

#[test]
fn missing_value_comes_first_only_where_the_facet_has_one() {
    let data = dataset();
    let options = english_options(&data);
    for facet in FacetId::ALL {
        let list = options.get(facet);
        let first = list.get(0);
        let has_missing = first.is_some_and(|option| option.value == OptionValue::Symbol(None));
        assert_eq!(has_missing, facet.has_missing_option(), "{facet:?}");
        if has_missing {
            assert_eq!(first.map(|option| option.key.as_str()), Some("!"));
            assert_eq!(list.label(0), "Missing");
            assert_eq!(list.find_key("!"), Some(OptionValue::Symbol(None)));
        } else {
            assert_ne!(list.find_key("!"), Some(OptionValue::Symbol(None)));
        }
    }
}

#[test]
fn raw_materials_use_plain_labels() {
    let data = dataset();
    let options = english_options(&data);
    let list = options.get(FacetId::RawMaterial);
    assert_eq!(labels_of(list), ["Missing", "goods_gold", "iron", "wheat"]);
    assert_eq!(list.position(OptionValue::Symbol(Some(WHEAT))), Some(3));
}

#[test]
fn find_key_resolves_every_option_and_rejects_unknown_keys() {
    let data = dataset();
    let options = english_options(&data);
    for facet in FacetId::ALL {
        let list = options.get(facet);
        for index in 0..list.len() {
            let Ok(index) = u32::try_from(index) else { continue };
            let Some(option) = list.get(index) else { continue };
            assert_eq!(list.find_key(&option.key), Some(option.value), "{facet:?} {}", option.key);
            assert_eq!(list.position(option.value), Some(index));
        }
        assert_eq!(list.find_key("no_such_key"), None);
        assert_eq!(list.find_key(""), None);
    }
    assert_eq!(options.get(FacetId::Topography).find_key("Hills"), None);
}

#[test]
fn find_key_works_on_unsorted_input() {
    let list = OptionList::new(vec![
        FacetOption::new(OptionValue::Flag(true), "zeta", "Z".to_owned()),
        FacetOption::new(OptionValue::Flag(false), "alpha", "A".to_owned()),
        FacetOption::new(OptionValue::Tier(4), "mid", "M".to_owned()),
    ]);
    assert_eq!(list.find_key("zeta"), Some(OptionValue::Flag(true)));
    assert_eq!(list.find_key("alpha"), Some(OptionValue::Flag(false)));
    assert_eq!(list.find_key("mid"), Some(OptionValue::Tier(4)));
    assert_eq!(list.find_key("alp"), None);
    assert_eq!(OptionList::default().find_key("alpha"), None);
    assert!(OptionList::default().is_empty());
}

#[test]
fn out_of_range_access_is_empty() {
    let data = dataset();
    let options = english_options(&data);
    let list = options.get(FacetId::Kind);
    assert!(list.get(99).is_none());
    assert_eq!(list.label(99), "");
    assert_eq!(list.value(99), None);
    assert_eq!(FacetOptions::default().get(FacetId::Kind).len(), 0);
}

#[test]
fn search_matches_diacritics_case_and_separators() {
    let list = free_list(&["Kouřim", "Stockholm", "Ra's al-'Ain", "Abu Dhabi"]);
    assert_eq!(search(&list, ""), [0, 1, 2, 3]);
    assert_eq!(search(&list, "kourim"), [0]);
    assert_eq!(search(&list, "KOUŘIM"), [0]);
    assert_eq!(search(&list, "ouř"), [0]);
    assert_eq!(search(&list, "hol"), [1]);
    for query in ["rasalain", "ras al ain", "ras_al_ain", "ras—al—ʿain", "RAS AL-AIN"] {
        assert_eq!(search(&list, query), [2], "{query:?}");
    }
    assert_eq!(search(&list, "abudhabi"), [3]);
    assert_eq!(search(&list, "zzz"), Vec::<u32>::new());
    // A query with only separators folds to nothing and keeps every option.
    assert_eq!(search(&list, " -- "), [0, 1, 2, 3]);
}

#[test]
fn search_matches_non_latin_text() {
    let list = free_list(&["Москва", "東京", "서울", "القاهرة", "Athens"]);
    assert_eq!(search(&list, "москва"), [0]);
    assert_eq!(search(&list, "МОСКВ"), [0]);
    assert_eq!(search(&list, "東"), [1]);
    assert_eq!(search(&list, "京"), [1]);
    assert_eq!(search(&list, "서울"), [2]);
    assert_eq!(search(&list, "قاهرة"), [3]);
    assert_eq!(search(&list, "москва東"), Vec::<u32>::new());
}

#[test]
fn search_replaces_the_output_and_keeps_display_order() {
    let list = free_list(&["Alpha", "Beta", "Alphabet", "Gamma"]);
    let mut out = vec![5, 6, 7, 8, 9];
    list.filter_into("alpha", &mut out);
    assert_eq!(out, [0, 2]);
    list.filter_into("", &mut out);
    assert_eq!(out, [0, 1, 2, 3]);
    list.filter_into("nothing", &mut out);
    assert!(out.is_empty());
}

#[test]
fn dataset_search_covers_label_and_key() {
    let data = dataset();
    let options = english_options(&data);
    let list = options.get(FacetId::Subcontinent);
    assert_eq!(search(list, "kourim"), [0]);
    assert_eq!(search(list, "kořim"), Vec::<u32>::new());
    assert_eq!(search(list, "KOUŘIM"), [0]);
    let area = options.get(FacetId::Area);
    assert_eq!(search(area, "москва"), [0]);
    assert_eq!(search(area, "moskva"), [0]);
    assert_eq!(search(options.get(FacetId::Province), "東京"), [0]);
}

#[test]
fn the_active_game_language_labels_the_options() {
    let data = dataset();
    let pack = korean_pack(&data);
    assert!(pack.is_some(), "the synthetic pack must validate");
    let Some(pack) = pack else { return };
    let options = FacetOptions::build(Labels::new(&data, Some(&pack)), Locale::EnUs);
    let list = options.get(FacetId::Topography);
    assert_eq!(labels_of(list), ["mountains", "ocean", "언덕 [hills]"]);
    assert_eq!(search(list, "언덕"), [2]);
    assert_eq!(search(list, "hills"), [2]);
    assert_eq!(keys_of(list), ["mountains", "ocean", "hills"]);
    assert_eq!(list.find_key("hills"), Some(OptionValue::Symbol(Some(HILLS))));
    assert_eq!(options.get(FacetId::RawMaterial).label(3), "밀");
    // A symbol the pack leaves alone keeps its English label.
    assert_eq!(options.get(FacetId::Subcontinent).get(0).map(|o| o.value), Some(OptionValue::Symbol(Some(KOURIM))));
    assert_eq!(options.get(FacetId::Subcontinent).label(0), "Kouřim [kourim]");
}
