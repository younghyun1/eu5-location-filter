//! Resolution against the fixture's option lists and location keys, and full round trips.
//!
//! These tests depend on `FacetOptions::build`, `OptionList::find_key` and `set_selected`
//! producing the documented wire keys (symbol identifiers, kind and presence words, tier digits,
//! `!` for the missing value).

use super::fixture::{BOREAL, EVORA, FOREST, HILLS, Loaded, MOUNTAINS, STOCKHOLM, dataset};
use crate::app::view_model::view_state::LocationKeys;
use crate::model::{LocationId, LocationKind};

#[test]
fn location_keys_find_by_binary_search() {
    let data = dataset();
    let keys = LocationKeys::build(&data);
    assert_eq!(keys.find(&data, "stockholm"), Some(LocationId(0)));
    assert_eq!(keys.find(&data, "baltic_sea"), Some(LocationId(1)));
    assert_eq!(keys.find(&data, "évora"), Some(LocationId(2)));
    assert_eq!(keys.find(&data, "Stockholm"), None);
    assert_eq!(keys.find(&data, ""), None);
    assert_eq!(LocationKeys::default().find(&data, "stockholm"), None);
    // Keys are symbols, not names: the dictionary still holds the key text.
    assert_eq!(data.symbol(STOCKHOLM), Some("stockholm"));
    assert_eq!(data.symbol(EVORA), Some("évora"));
}

#[test]
fn kind_defaults_follow_the_three_states() {
    let loaded = Loaded::new();
    let kinds = |text: &str| loaded.resolve(text).map(|view| view.filters.kinds);
    assert_eq!(kinds("s1").map(|set| set.len()), Some(1));
    assert!(kinds("s1").is_some_and(|set| set.contains(&LocationKind::Land)));
    assert!(kinds("s1~k.").is_some_and(|set| set.is_empty()));
    assert!(kinds("s1~k.foo").is_some_and(|set| set.len() == 1 && set.contains(&LocationKind::Land)));
    assert!(kinds("s1~k.sea*lake").is_some_and(|set| set.len() == 2 && !set.contains(&LocationKind::Land)));
}

#[test]
fn facet_items_resolve_through_option_keys() {
    let loaded = Loaded::new();
    let view = loaded.resolve("s1~tp.hills*mountains*nowhere~vg.forest~cl.!*boreal~lm.2*3*9");
    assert!(view.is_some());
    if let Some(view) = view {
        let filters = &view.filters;
        assert!(filters.topographies.len() == 2 && filters.topographies.contains(&HILLS));
        assert!(filters.topographies.contains(&MOUNTAINS));
        assert!(filters.vegetation.len() == 1 && filters.vegetation.contains(&Some(FOREST)));
        assert!(filters.climates.contains(&None) && filters.climates.contains(&Some(BOREAL)));
        // Tier 9 is above the highest river tier and has no option.
        assert!(filters.river_level_min.len() == 2 && filters.river_level_min.contains(&2));
        assert!(filters.river_level_min.contains(&3));
    }
}

#[test]
fn escaped_bang_never_selects_the_missing_value() {
    let loaded = Loaded::new();
    let view = loaded.resolve("s1~vg.!21~cl.!");
    assert!(view.as_ref().is_some_and(|view| view.filters.vegetation.is_empty()));
    assert!(view.is_some_and(|view| view.filters.climates.contains(&None)));
}

#[test]
fn selection_resolves_by_key() {
    let loaded = Loaded::new();
    let selected = |text: &str| loaded.resolve(text).and_then(|view| view.selected);
    assert_eq!(selected("s1~l.stockholm"), Some(LocationId(0)));
    assert_eq!(selected("s1~l.!C3!A9vora"), Some(LocationId(2)));
    assert_eq!(selected("s1~l.atlantis"), None);
    assert_eq!(selected("s1"), None);
}

#[test]
fn non_canonical_input_reports_the_canonical_string() {
    let loaded = Loaded::new();
    assert_eq!(
        loaded.canonical("s1~k.sea*land~zz.9~q.river~q.lake").as_deref(),
        Some("s1~q.river~k.land*sea")
    );
    assert_eq!(loaded.canonical("s1~s.rgb*a~k.land").as_deref(), Some("s1~s.color*a"));
    assert_eq!(loaded.canonical("s1~hr.-0*~lm.03*3").as_deref(), Some("s1~lm.3~hr.0*"));
    assert_eq!(loaded.canonical("s2~q.river"), None);
}

#[test]
fn specification_examples_round_trip() {
    let loaded = Loaded::new();
    for text in [
        "s1",
        "s1~q.river~k.land*sea~s.harbor*d",
        "s1~q.new!20york~tp.hills*mountains~cl.!~si.0",
        "s1~q.!EC!84!9C!EC!9A!B8~c.1A2B3C~co.yes~lm.2*3~hr.0.5*",
        "s1~k.~rm.!*wheat~fo.1~mx.-1.5*2~s.name*d~l.stockholm",
        "s1~k.impassable*land~ct.europe~pv.uppland~rl.catholic~rp.present~hp.missing~mp.present",
    ] {
        assert_eq!(loaded.canonical(text).as_deref(), Some(text), "{text}");
    }
}
