//! Whole-string checks, per-field and per-item drop rules, and the specification examples.

use super::super::parse;
use crate::app::view_model::view_state::{
    RawItem, RawViewState, VIEW_STATE_MAX_LEN, decode, passes_whole_checks,
};
use crate::app::view_model::{FacetId, SortState};
use crate::filter::{FloatRange, SortField};
use crate::model::MapColor;

fn raw(text: &str) -> RawViewState {
    let decoded = decode(text);
    assert!(decoded.is_ok(), "{text} was rejected");
    decoded.unwrap_or_default()
}

fn facet(state: &RawViewState, facet: FacetId) -> Option<Vec<RawItem>> {
    state.facets.get(facet.index()).cloned().flatten()
}

fn key(text: &str) -> RawItem {
    RawItem::Key(text.to_owned())
}

#[test]
fn whole_string_checks() {
    let too_long = format!("s1~q.{}", "a".repeat(VIEW_STATE_MAX_LEN - 4));
    assert_eq!(too_long.len(), VIEW_STATE_MAX_LEN + 1);
    for rejected in [
        "",
        "s2",
        "S1",
        "s1x",
        " s1",
        "s10~q.a",
        "s1~q.a b",
        "s1~q.%20",
        "s1~q.\u{e9}",
        "q.river~s1",
    ] {
        assert!(!passes_whole_checks(rejected), "{rejected:?}");
        assert!(decode(rejected).is_err(), "{rejected:?}");
    }
    assert!(!passes_whole_checks(&too_long));
    assert!(decode(&too_long).is_err());
    let at_limit = format!("s1~q.{}", "a".repeat(VIEW_STATE_MAX_LEN - 5));
    assert!(passes_whole_checks(&at_limit));
    for accepted in ["s1", "s1~", "s1~q", "s1~~~", "s1~q.a"] {
        assert!(passes_whole_checks(accepted), "{accepted:?}");
    }
}

#[test]
fn empty_and_nameless_tokens_leave_defaults() {
    for text in ["s1", "s1~", "s1~q", "s1~~q~.", "s1~zz.9"] {
        assert_eq!(raw(text), RawViewState::default(), "{text}");
    }
}

#[test]
fn invalid_search_text_takes_the_default() {
    for text in [
        "s1~q.!2", "s1~q.!FF", "s1~q.!00", "s1~q.!1F", "s1~q.!7F", "s1~q.", "s1~q.!",
    ] {
        assert_eq!(raw(text).search, "", "{text}");
    }
    let over = format!("s1~q.{}", "a".repeat(257));
    assert_eq!(raw(&over).search, "");
    let limit = format!("s1~q.{}", "a".repeat(256));
    assert_eq!(raw(&limit).search.len(), 256);
}

#[test]
fn colour_needs_six_hex_digits() {
    assert_eq!(raw("s1~c.12345").rgb, None);
    assert_eq!(raw("s1~c.1234567").rgb, None);
    assert_eq!(raw("s1~c.12345G").rgb, None);
    assert_eq!(raw("s1~c.1a2b3c").rgb, Some(MapColor(0x1A_2B3C)));
}

#[test]
fn ranges_follow_the_number_grammar() {
    let harbor = |text: &str| raw(&format!("s1~hr.{text}")).harbor_range;
    for bad in [
        "1e5*", "--1*", "-*", ".5*", "1.*", "inf*", "NaN*", "1", "1*2*3", "*0x1", "1_0*",
    ] {
        assert_eq!(harbor(bad), FloatRange::default(), "{bad}");
    }
    assert_eq!(
        harbor("0.5*"),
        FloatRange {
            min: Some(0.5),
            max: None
        }
    );
    assert_eq!(
        harbor("*-2"),
        FloatRange {
            min: None,
            max: Some(-2.0)
        }
    );
    assert_eq!(
        harbor("-1.5*2"),
        FloatRange {
            min: Some(-1.5),
            max: Some(2.0)
        }
    );
    let zero = harbor("-0*").min.unwrap_or(f32::NAN);
    assert_eq!(zero.to_bits(), 0.0_f32.to_bits());
    let overflow = format!("{}*", "9".repeat(40));
    assert_eq!(harbor(&overflow), FloatRange::default());
    let long = format!("0.{}*", "1".repeat(47));
    assert_eq!(long.len(), 50);
    assert_eq!(harbor(&long), FloatRange::default());
    let limit = format!("0.{}*", "1".repeat(46));
    assert!(harbor(&limit).min.is_some());
}

#[test]
fn numbers_reject_what_str_parse_would_accept() {
    for bad in [
        "+1", ".5", "1.", "1e5", "1E5", "inf", "-inf", "nan", "NaN", "", "-", "0x10", " 1", "1_0",
    ] {
        assert_eq!(parse::number(bad), None, "{bad:?}");
    }
    assert_eq!(parse::number("007"), Some(7.0));
    assert_eq!(parse::number("-1.25"), Some(-1.25));
    // 39 nines exceed f32::MAX and parse to infinity.
    assert_eq!(parse::number(&"9".repeat(39)), None);
    assert_eq!(parse::number(&format!("1{}", "0".repeat(38))), Some(1e38));
}

#[test]
fn sort_flags_and_selection() {
    assert_eq!(raw("s1~s.name*x").sort, SortState::default());
    assert_eq!(raw("s1~s.bogus*a").sort, SortState::default());
    assert_eq!(
        raw("s1~s.rgb*d").sort,
        SortState {
            field: SortField::Color,
            ascending: false
        }
    );
    assert_eq!(
        raw("s1~s.harbor*d").sort.field,
        SortField::HarborSuitability
    );
    assert!(!raw("s1~si.0").show_impassable);
    assert!(raw("s1~si.2").show_impassable);
    assert!(raw("s1~fo.1").food_only);
    assert!(!raw("s1~fo.yes").food_only);
    assert_eq!(raw("s1~l.stockholm").selected.as_deref(), Some("stockholm"));
    assert_eq!(raw("s1~l.").selected, None);
    let over = format!("s1~l.{}", "a".repeat(129));
    assert_eq!(raw(&over).selected, None);
}

#[test]
fn list_items_follow_the_facet_rules() {
    assert_eq!(facet(&raw("s1~tp.!"), FacetId::Topography), None);
    assert_eq!(
        facet(&raw("s1~ct.!*europe"), FacetId::Continent),
        Some(vec![key("europe")])
    );
    assert_eq!(
        facet(&raw("s1~vg.!"), FacetId::Vegetation),
        Some(vec![RawItem::Missing])
    );
    assert_eq!(
        facet(&raw("s1~vg.!21"), FacetId::Vegetation),
        Some(vec![key("!")])
    );
    assert_eq!(facet(&raw("s1~co.!"), FacetId::Coastal), None);
    assert_eq!(
        facet(&raw("s1~co.maybe*yes"), FacetId::Coastal),
        Some(vec![key("yes")])
    );
    assert_eq!(
        facet(&raw("s1~hp.missing"), FacetId::HarborPresence),
        Some(vec![key("missing")])
    );
    assert_eq!(facet(&raw("s1~lm.256"), FacetId::RiverLevelMin), None);
    assert_eq!(
        facet(&raw("s1~lm.0*01*-1*3"), FacetId::RiverLevelMin),
        Some(vec![key("3")])
    );
    assert_eq!(
        facet(&raw("s1~lx.255"), FacetId::RiverLevelMax),
        Some(vec![key("255")])
    );
    assert_eq!(
        facet(&raw("s1~tp.a**b"), FacetId::Topography),
        Some(vec![key("a"), key("b")])
    );
    assert_eq!(
        facet(&raw("s1~tp.!2*b"), FacetId::Topography),
        Some(vec![key("b")])
    );
}

#[test]
fn kind_distinguishes_absent_empty_and_dropped() {
    assert_eq!(facet(&raw("s1"), FacetId::Kind), None);
    assert_eq!(facet(&raw("s1~k."), FacetId::Kind), Some(Vec::new()));
    assert_eq!(facet(&raw("s1~k.foo"), FacetId::Kind), None);
    assert_eq!(
        facet(&raw("s1~k.land*bogus*sea"), FacetId::Kind),
        Some(vec![key("land"), key("sea")])
    );
}

#[test]
fn first_occurrence_wins_even_when_invalid() {
    assert_eq!(raw("s1~q.river~q.lake").search, "river");
    assert_eq!(raw("s1~q.!~q.lake").search, "");
    assert_eq!(
        facet(&raw("s1~k.sea~k.lake"), FacetId::Kind),
        Some(vec![key("sea")])
    );
}

#[test]
fn reads_at_most_64_fields_and_1000_items() {
    let filler = "~zz.1".repeat(63);
    assert_eq!(raw(&format!("s1{filler}~q.a")).search, "a");
    assert_eq!(raw(&format!("s1{filler}~zz.1~q.a")).search, "");
    let read = format!("s1~tp.{}a", "*".repeat(999));
    assert_eq!(
        facet(&raw(&read), FacetId::Topography),
        Some(vec![key("a")])
    );
    let ignored = format!("s1~tp.{}a", "*".repeat(1_000));
    assert_eq!(facet(&raw(&ignored), FacetId::Topography), None);
}

#[test]
fn specification_examples_decode() {
    let second = raw("s1~q.river~k.land*sea~s.harbor*d");
    assert_eq!(second.search, "river");
    assert_eq!(
        facet(&second, FacetId::Kind),
        Some(vec![key("land"), key("sea")])
    );
    assert_eq!(
        second.sort,
        SortState {
            field: SortField::HarborSuitability,
            ascending: false
        }
    );

    let third = raw("s1~q.new!20york~tp.hills*mountains~cl.!~si.0");
    assert_eq!(third.search, "new york");
    assert_eq!(
        facet(&third, FacetId::Topography),
        Some(vec![key("hills"), key("mountains")])
    );
    assert_eq!(
        facet(&third, FacetId::Climate),
        Some(vec![RawItem::Missing])
    );
    assert!(!third.show_impassable);

    let fourth = raw("s1~q.!EC!84!9C!EC!9A!B8~c.1A2B3C~co.yes~lm.2*3~hr.0.5*");
    assert_eq!(fourth.search, "서울");
    assert_eq!(fourth.rgb, Some(MapColor(0x1A_2B3C)));
    assert_eq!(facet(&fourth, FacetId::Coastal), Some(vec![key("yes")]));
    assert_eq!(
        facet(&fourth, FacetId::RiverLevelMin),
        Some(vec![key("2"), key("3")])
    );
    assert_eq!(
        fourth.harbor_range,
        FloatRange {
            min: Some(0.5),
            max: None
        }
    );

    let fifth = raw("s1~k.~rm.!*wheat~fo.1~mx.-1.5*2~s.name*d~l.stockholm");
    assert_eq!(facet(&fifth, FacetId::Kind), Some(Vec::new()));
    assert_eq!(
        facet(&fifth, FacetId::RawMaterial),
        Some(vec![RawItem::Missing, key("wheat")])
    );
    assert!(fifth.food_only);
    assert_eq!(
        fifth.movement_x,
        FloatRange {
            min: Some(-1.5),
            max: Some(2.0)
        }
    );
    assert_eq!(
        fifth.sort,
        SortState {
            field: SortField::Name,
            ascending: false
        }
    );
    assert_eq!(fifth.selected.as_deref(), Some("stockholm"));

    let messy = raw("s1~k.sea*land~zz.9~q.river~q.lake");
    assert_eq!(messy.search, "river");
    assert_eq!(
        facet(&messy, FacetId::Kind),
        Some(vec![key("sea"), key("land")])
    );
}
