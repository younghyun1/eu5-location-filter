//! The view-state specification examples decoded end to end.

use super::decode_tests::{facet, key, raw};
use crate::app::view_model::view_state::RawItem;
use crate::app::view_model::{FacetId, SortState};
use crate::filter::{FloatRange, SortField};
use crate::model::MapColor;

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
