//! Tests of the rail's section plan and of building the rail in every mode and state.

use iced::Element;

use super::RailModel;
use super::rail::view;
use super::sections::{Slot, slots};
use crate::app::context::{Platform, ViewContext};
use crate::app::i18n::{Locale, Text};
use crate::app::layout::{Direction, LayoutMode};
use crate::app::message::Message;
use crate::app::state::{FieldBuffer, TextFields};
use crate::app::theme::PALETTE_DARK;
use crate::app::view_model::{Bound, FacetId, FacetUi, RailSection, RangeField};
use crate::filter::FilterSet;

fn context(direction: Direction, mode: LayoutMode) -> ViewContext {
    ViewContext {
        palette: PALETTE_DARK,
        locale: Locale::EnUs,
        direction,
        mode,
        platform: Platform::Desktop,
    }
}

#[test]
fn every_facet_appears_once_in_rail_order() {
    let facets: Vec<FacetId> = RailSection::ALL
        .into_iter()
        .flat_map(slots)
        .filter_map(|slot| match slot {
            Slot::Facet(facet) => Some(facet),
            _ => None,
        })
        .collect();
    assert_eq!(facets, FacetId::ALL);
}

#[test]
fn non_facet_controls_sit_where_the_contract_puts_them() {
    assert_eq!(
        slots(RailSection::Location),
        [Slot::Facet(FacetId::Kind), Slot::ShowImpassable]
    );
    assert_eq!(
        slots(RailSection::Economy),
        [
            Slot::Facet(FacetId::RawMaterial),
            Slot::FoodOnly,
            Slot::Facet(FacetId::Modifier)
        ]
    );
    assert_eq!(
        slots(RailSection::MapData),
        [
            Slot::Rgb,
            Slot::Facet(FacetId::Coastal),
            Slot::Facet(FacetId::RiverPresence),
            Slot::Facet(FacetId::RiverLevelMin),
            Slot::Facet(FacetId::RiverLevelMax),
            Slot::Facet(FacetId::HarborPresence),
            Slot::Range(RangeField::Harbor),
            Slot::Facet(FacetId::MovementPresence),
            Slot::Range(RangeField::MovementX),
            Slot::Range(RangeField::MovementY),
        ]
    );
}

#[test]
fn geography_terrain_and_society_hold_only_facets() {
    for section in [
        RailSection::Geography,
        RailSection::Terrain,
        RailSection::Society,
    ] {
        assert!(
            slots(section)
                .iter()
                .all(|slot| matches!(slot, Slot::Facet(_)))
        );
    }
}

#[test]
fn rail_builds_in_every_mode_state_and_direction() {
    let filters = FilterSet::land_only();
    let facets = FacetId::ALL.map(FacetUi::new);
    let mut fields = TextFields::default();
    fields.rgb = FieldBuffer {
        text: "zz".to_owned(),
        error: Some(Text::ValidationRgb),
    };
    fields.range_mut(RangeField::Harbor, Bound::Max).error = Some(Text::ValidationNumber);
    for direction in [Direction::Ltr, Direction::Rtl] {
        for mode in [LayoutMode::Wide, LayoutMode::Narrow] {
            for (collapsed, in_drawer) in [(false, false), (true, false), (false, true)] {
                let model = RailModel {
                    filters: &filters,
                    options: None,
                    facets: &facets,
                    fields: &fields,
                };
                let ctx = context(direction, mode);
                let _: Element<'_, Message> = view(ctx, model, collapsed, in_drawer);
            }
        }
    }
}
