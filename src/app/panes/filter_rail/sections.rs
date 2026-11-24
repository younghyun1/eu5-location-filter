//! The scrolling content of the rail: the six sections, in the order and with the non-facet
//! controls the contract fixes.

use std::sync::OnceLock;

use iced::widget::{Column, container, space};
use iced::{Element, Length, Padding};

use super::facet_group;
use super::fields::{range_row, rgb_field};
use super::rail::RailModel;
use crate::app::context::ViewContext;
use crate::app::layout::{
    RAIL_GROUP_GAP, RAIL_PADDING_BLOCK, RAIL_PADDING_INLINE, RAIL_SECTION_GAP, SECTION_TITLE_HEIGHT,
};
use crate::app::message::{FilterMessage, Message};
use crate::app::panes::controls::{caption, toggle_row};
use crate::app::view_model::{FacetId, OptionList, RailSection, RangeField};

/// One control of a rail section.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Slot {
    /// A facet group.
    Facet(FacetId),
    /// The Show impassable toggle.
    ShowImpassable,
    /// The Food-producing only toggle.
    FoodOnly,
    /// The exact-colour field.
    Rgb,
    /// A numeric range row.
    Range(RangeField),
}

/// Controls of `section` in rail order: the facet groups of the section, with the toggles and
/// fields that belong after or before them.
#[must_use]
pub(super) fn slots(section: RailSection) -> Vec<Slot> {
    let mut out = Vec::new();
    if section == RailSection::MapData {
        out.push(Slot::Rgb);
    }
    for facet in FacetId::ALL {
        if facet.section() != section {
            continue;
        }
        out.push(Slot::Facet(facet));
        match facet {
            FacetId::Kind => out.push(Slot::ShowImpassable),
            FacetId::RawMaterial => out.push(Slot::FoodOnly),
            FacetId::HarborPresence => out.push(Slot::Range(RangeField::Harbor)),
            FacetId::MovementPresence => {
                out.push(Slot::Range(RangeField::MovementX));
                out.push(Slot::Range(RangeField::MovementY));
            }
            _ => {}
        }
    }
    out
}

/// Option list shown before data loads, when groups have no options.
fn empty_options() -> &'static OptionList {
    static EMPTY: OnceLock<OptionList> = OnceLock::new();
    EMPTY.get_or_init(OptionList::default)
}

/// All sections in a column with the rail's content padding.
#[must_use]
pub(super) fn content<'a>(ctx: ViewContext, rail: RailModel<'a>) -> Element<'a, Message> {
    let mut column = Column::new()
        .width(Length::Fill)
        .padding(Padding::from([RAIL_PADDING_BLOCK, RAIL_PADDING_INLINE]));
    for section in RailSection::ALL {
        column = column.push(section_view(ctx, rail, section));
    }
    column.into()
}

/// One section: the gap above it, its title, and its controls separated by `RAIL_GROUP_GAP`.
fn section_view<'a>(
    ctx: ViewContext,
    rail: RailModel<'a>,
    section: RailSection,
) -> Element<'a, Message> {
    let mut controls = Column::new().spacing(RAIL_GROUP_GAP).width(Length::Fill);
    for slot in slots(section) {
        if let Some(control) = slot_view(ctx, rail, slot) {
            controls = controls.push(control);
        }
    }
    let title = container(caption(ctx, ctx.t(section.title())))
        .width(Length::Fill)
        .align_x(ctx.direction.start())
        .center_y(SECTION_TITLE_HEIGHT);
    Column::new()
        .push(space().height(RAIL_SECTION_GAP))
        .push(title)
        .push(controls)
        .width(Length::Fill)
        .into()
}

/// The element of one slot; a facet missing from the state array draws nothing.
fn slot_view<'a>(
    ctx: ViewContext,
    rail: RailModel<'a>,
    slot: Slot,
) -> Option<Element<'a, Message>> {
    match slot {
        Slot::Facet(facet) => {
            let ui = rail.facets.get(facet.index())?;
            let options = match rail.options {
                Some(options) => options.get(facet),
                None => empty_options(),
            };
            Some(facet_group::view(ctx, ui, options, rail.filters))
        }
        Slot::ShowImpassable => Some(toggle_row(
            ctx,
            ctx.t(crate::app::i18n::Text::RailShowImpassable),
            rail.filters.show_impassable,
            |on| Message::Filter(FilterMessage::ShowImpassable(on)),
        )),
        Slot::FoodOnly => Some(toggle_row(
            ctx,
            ctx.t(crate::app::i18n::Text::RailFoodOnly),
            rail.filters.food_producing_only,
            |on| Message::Filter(FilterMessage::FoodOnly(on)),
        )),
        Slot::Rgb => Some(rgb_field(ctx, &rail.fields.rgb)),
        Slot::Range(field) => Some(range_row(ctx, field, rail.fields)),
    }
}
