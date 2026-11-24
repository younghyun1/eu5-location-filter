//! Facet checklists in the filter rail: toggling an option, unfolding a group, searching its
//! options, and the badge state that results.

use super::fixture::{EUROPE_LAND_COUNT, LAND_COUNT, SEA_COUNT};
use super::harness::{Harness, WIDE};
use super::selectors::{has_id, text_in, text_selector};
use crate::AppError;
use crate::app::i18n::{Locale, Text, t};
use crate::app::ids;
use crate::app::message::{FilterMessage, Message};
use crate::app::view_model::{FacetId, OptionValue};
use crate::model::LocationKind;

/// Label of the first option of `facet` whose text contains `needle`.
fn option_label(harness: &Harness, facet: FacetId, needle: &str) -> Result<String, AppError> {
    let Some(data) = harness.app.data.as_ref() else {
        return Err(AppError::InvalidData("no dataset is installed".to_owned()));
    };
    let list = data.options.get(facet);
    let found = (0..list.len())
        .filter_map(|index| u32::try_from(index).ok())
        .map(|index| list.label(index))
        .find(|label| label.contains(needle));
    match found {
        Some(label) => Ok(label.to_owned()),
        None => Err(AppError::InvalidData(format!("no option containing {needle:?}"))),
    }
}

#[test]
fn toggling_the_sea_option_adds_the_sea_locations() -> Result<(), AppError> {
    let mut harness = Harness::new(WIDE)?;
    let sea = t(Locale::EnUs, Text::OptionKindSea);

    let messages = harness.drive(|ui| {
        let row = text_selector(ui, ids::facet_list(FacetId::Kind), sea)?;
        let _target = ui.click(row)?;
        Ok(())
    })?;

    assert!(
        messages.iter().any(|message| matches!(
            message,
            Message::Filter(FilterMessage::Toggle {
                facet: FacetId::Kind,
                value: OptionValue::Kind(LocationKind::Sea),
                selected: true,
            })
        )),
        "{messages:?}"
    );
    assert!(harness.app.filters.kinds.contains(&LocationKind::Sea));
    assert_eq!(harness.results()?.len(), LAND_COUNT + SEA_COUNT);
    let mut ui = harness.ui();
    assert!(ui.find(harness.count_text(LAND_COUNT + SEA_COUNT)).is_ok());
    Ok(())
}

#[test]
fn toggling_a_selected_option_clears_it() -> Result<(), AppError> {
    let mut harness = Harness::new(WIDE)?;
    harness.send(Message::Filter(FilterMessage::Toggle {
        facet: FacetId::Kind,
        value: OptionValue::Kind(LocationKind::Sea),
        selected: true,
    }));
    let sea = t(Locale::EnUs, Text::OptionKindSea);

    let messages = harness.drive(|ui| {
        let row = text_selector(ui, ids::facet_list(FacetId::Kind), sea)?;
        let _target = ui.click(row)?;
        Ok(())
    })?;

    assert!(
        messages.iter().any(|message| matches!(
            message,
            Message::Filter(FilterMessage::Toggle {
                value: OptionValue::Kind(LocationKind::Sea),
                selected: false,
                ..
            })
        )),
        "{messages:?}"
    );
    assert!(!harness.app.filters.kinds.contains(&LocationKind::Sea));
    assert_eq!(harness.results()?.len(), LAND_COUNT);
    Ok(())
}

#[test]
fn unfolding_a_group_and_choosing_an_option_filters_by_it() -> Result<(), AppError> {
    let mut harness = Harness::new(WIDE)?;
    let continent = FacetId::Continent;
    assert!(harness.app.facets.get(continent.index()).is_some_and(|group| group.folded));
    assert!(!has_id(&mut harness.ui(), ids::facet_list(continent)), "a folded group has no list");

    let messages = harness.drive(|ui| {
        let _fold = ui.click(ids::facet_fold(continent))?;
        Ok(())
    })?;
    assert!(
        messages.iter().any(|message| matches!(
            message,
            Message::Filter(FilterMessage::Fold { facet: FacetId::Continent, folded: false })
        )),
        "{messages:?}"
    );
    assert!(harness.app.facets.get(continent.index()).is_some_and(|group| !group.folded));

    let europe = option_label(&harness, continent, "europe")?;
    harness.drive(|ui| {
        let row = text_selector(ui, ids::facet_list(continent), &europe)?;
        let _target = ui.click(row)?;
        Ok(())
    })?;

    assert_eq!(harness.app.filters.continents.len(), 1);
    assert_eq!(harness.results()?.len(), EUROPE_LAND_COUNT);
    Ok(())
}

#[test]
fn clear_all_empties_every_facet_selection() -> Result<(), AppError> {
    let mut harness = Harness::new(WIDE)?;
    harness.send(Message::Filter(FilterMessage::Toggle {
        facet: FacetId::Kind,
        value: OptionValue::Kind(LocationKind::Sea),
        selected: true,
    }));
    harness.send(Message::Filter(FilterMessage::OptionQuery {
        facet: FacetId::Kind,
        query: "sea".to_owned(),
    }));
    assert_eq!(harness.results()?.len(), LAND_COUNT + SEA_COUNT);

    harness.drive(|ui| {
        let _button = ui.click(ids::RAIL_CLEAR_ALL)?;
        Ok(())
    })?;

    assert_eq!(harness.results()?.len(), LAND_COUNT);
    let kinds = &harness.app.filters.kinds;
    assert!(kinds.len() == 1 && kinds.contains(&LocationKind::Land));
    assert!(harness.app.facets.get(FacetId::Kind.index()).is_some_and(|group| group.query.is_empty()));
    Ok(())
}

#[test]
fn the_option_search_hides_options_without_changing_the_filter() -> Result<(), AppError> {
    let mut harness = Harness::new(WIDE)?;
    let kind = FacetId::Kind;

    harness.drive(|ui| {
        let _field = ui.click(ids::facet_search(kind))?;
        let _status = ui.typewrite("sea");
        Ok(())
    })?;

    let query = harness.app.facets.get(kind.index()).map(|group| group.query.as_str());
    assert_eq!(query, Some("sea"));
    assert_eq!(harness.results()?.len(), LAND_COUNT, "the filter itself is unchanged");
    let mut ui = harness.ui();
    assert!(text_in(&mut ui, ids::facet_list(kind), t(Locale::EnUs, Text::OptionKindSea)).is_ok());
    assert!(text_in(&mut ui, ids::facet_list(kind), t(Locale::EnUs, Text::OptionKindLand)).is_err());
    Ok(())
}
