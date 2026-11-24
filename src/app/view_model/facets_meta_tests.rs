//! Facet metadata, range rows, facet row rendering and sort tokens.

use super::options::test_data::{dataset, english_options};
use super::{
    Bound, FacetId, FacetRows, FacetUi, OptionValue, RailSection, RangeField, SortState,
    column_for_sort, set_selected, sort_from_token, sort_token,
};
use crate::app::widget::ChecklistModel;
use crate::filter::{FilterSet, FloatRange, SortField};
use crate::model::SymbolId;

#[test]
fn facet_metadata_is_consistent() {
    for (position, facet) in FacetId::ALL.into_iter().enumerate() {
        assert_eq!(facet.index(), position);
        assert_eq!(facet.starts_folded(), facet != FacetId::Kind);
    }
    let mut keys: Vec<&str> = FacetId::ALL.iter().map(|facet| facet.key()).collect();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(keys.len(), FacetId::COUNT);
    let missing: Vec<FacetId> = FacetId::ALL
        .into_iter()
        .filter(|f| f.has_missing_option())
        .collect();
    assert_eq!(missing.len(), 6);
    assert_eq!(FacetId::Kind.section(), RailSection::Location);
    assert_eq!(FacetId::Province.section(), RailSection::Geography);
    assert_eq!(FacetId::Climate.section(), RailSection::Terrain);
    assert_eq!(FacetId::Culture.section(), RailSection::Society);
    assert_eq!(FacetId::Modifier.section(), RailSection::Economy);
    assert_eq!(FacetId::MovementPresence.section(), RailSection::MapData);
    // Sections never go backwards along the rail order.
    let order = |section: RailSection| RailSection::ALL.iter().position(|s| *s == section);
    assert!(
        FacetId::ALL
            .windows(2)
            .all(|pair| order(pair[0].section()) <= order(pair[1].section()))
    );
}

#[test]
fn range_rows_read_and_write_their_filter_field() {
    let mut filters = FilterSet::default();
    let range = FloatRange {
        min: Some(1.0),
        max: Some(2.0),
    };
    for field in RangeField::ALL {
        assert_eq!(field.get(&filters), FloatRange::default());
        *field.get_mut(&mut filters) = range;
        assert_eq!(field.get(&filters), range);
    }
    assert_eq!(filters.harbor_range, range);
    assert_eq!(filters.movement_x, range);
    assert_eq!(filters.movement_y, range);
    assert_eq!((Bound::Min.index(), Bound::Max.index()), (0, 1));
    assert_eq!(RangeField::MovementY.index(), 2);
}

#[test]
fn facet_rows_show_visible_options_with_checked_state() {
    let data = dataset();
    let options = english_options(&data);
    let list = options.get(FacetId::Vegetation);
    let mut ui = FacetUi::new(FacetId::Vegetation);
    assert!(ui.folded && ui.query.is_empty() && ui.visible.is_empty());
    ui.refresh(list);
    assert_eq!(ui.visible.len(), list.len());
    let mut filters = FilterSet::default();
    let forest = list.position(OptionValue::Symbol(Some(SymbolId(9))));
    assert!(forest.is_some());
    if let Some(index) = forest {
        assert!(set_selected(
            &mut filters,
            FacetId::Vegetation,
            OptionValue::Symbol(Some(SymbolId(9))),
            true
        ));
        let rows = FacetRows {
            facet: FacetId::Vegetation,
            options: list,
            visible: &ui.visible,
            filters: &filters,
        };
        assert_eq!(rows.row_count(), list.len());
        let checked: Vec<usize> = (0..rows.row_count())
            .filter(|row| rows.is_checked(*row))
            .collect();
        assert_eq!(checked.len(), 1);
        assert_eq!(checked.first().map(|row| rows.option(*row)), Some(index));
        assert_eq!(
            rows.label(checked.first().copied().unwrap_or(usize::MAX)),
            list.label(index)
        );
    }
    ui.query = "forest".to_owned();
    ui.refresh(list);
    assert_eq!(ui.visible.len(), 1);
    let rows = FacetRows {
        facet: FacetId::Vegetation,
        options: list,
        visible: &ui.visible,
        filters: &filters,
    };
    assert!(rows.is_checked(0));
    assert_eq!(rows.label(5), "");
    assert!(!rows.is_checked(5));
    assert_eq!(rows.option(5), u32::MAX);
}

#[test]
fn sort_tokens_round_trip_and_toggle() {
    for field in SortField::ALL {
        assert_eq!(sort_from_token(sort_token(field)), Some(field), "{field:?}");
        assert_eq!(column_for_sort(field).sort_field(), field, "{field:?}");
    }
    assert_eq!(sort_from_token("rgb"), Some(SortField::Color));
    assert_eq!(sort_from_token("bogus"), None);
    assert_eq!(sort_from_token(""), None);
    let state = SortState::default();
    assert_eq!((state.field, state.ascending), (SortField::Name, true));
    let reversed = state.toggled(SortField::Name);
    assert!(!reversed.ascending && reversed.field == SortField::Name);
    let other = reversed.toggled(SortField::Kind);
    assert!(other.ascending && other.field == SortField::Kind);
}
