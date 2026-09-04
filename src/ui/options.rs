//! Dataset-backed options for searchable checkbox filters.

use std::collections::HashSet;

use memchr::memmem::Finder;
use slint::{Model, ModelRc, SharedString};

use super::{AppWindow, CheckOption};
use crate::filter::fold_search;
use crate::model::MAX_RIVER_LEVEL;
use crate::model::{Dataset, SymbolId};

pub(super) fn install(app: &AppWindow, dataset: &Dataset) {
    app.set_kind_options(static_model(&[
        ("land", "Land"),
        ("sea", "Sea"),
        ("lake", "Lake"),
        ("impassable", "Impassable"),
        ("unknown", "Unknown"),
    ]));
    app.set_continent_options(model(
        dataset,
        symbols(dataset, |r| Some(r.hierarchy.continent)),
        false,
    ));
    app.set_subcontinent_options(model(
        dataset,
        symbols(dataset, |r| Some(r.hierarchy.subcontinent)),
        false,
    ));
    app.set_region_options(model(
        dataset,
        symbols(dataset, |r| Some(r.hierarchy.region)),
        false,
    ));
    app.set_area_options(model(
        dataset,
        symbols(dataset, |r| Some(r.hierarchy.area)),
        false,
    ));
    app.set_province_options(model(
        dataset,
        symbols(dataset, |r| Some(r.hierarchy.province)),
        false,
    ));
    app.set_topography_options(model(
        dataset,
        symbols(dataset, |r| Some(r.topography)),
        false,
    ));
    app.set_vegetation_options(model(dataset, symbols(dataset, |r| r.vegetation), true));
    app.set_climate_options(model(dataset, symbols(dataset, |r| r.climate), true));
    app.set_religion_options(model(dataset, symbols(dataset, |r| r.religion), true));
    app.set_culture_options(model(dataset, symbols(dataset, |r| r.culture), true));
    app.set_raw_material_options(model(dataset, symbols(dataset, |r| r.raw_material), true));
    app.set_modifier_options(model(dataset, symbols(dataset, |r| r.modifier), true));
    app.set_coastal_options(static_model(&[("yes", "Yes"), ("no", "No")]));
    let presence = &[("present", "Present"), ("missing", "Missing")];
    app.set_river_options(static_model(presence));
    app.set_harbor_options(static_model(presence));
    app.set_movement_options(static_model(presence));
    let levels: Vec<CheckOption> = (1..=MAX_RIVER_LEVEL)
        .map(|level| option(&level.to_string(), &river_label(level)))
        .collect();
    app.set_river_level_options(ModelRc::from(levels.as_slice()));
}

fn river_label(level: u8) -> String {
    let name = crate::model::RiverLevel(level).label();
    format!("Level {level}: {name} (+{}%)", level * 10)
}

pub(super) fn filtered(
    source: ModelRc<CheckOption>,
    query: &str,
    mut checked: impl FnMut(&str) -> bool,
) -> ModelRc<CheckOption> {
    let query = fold_search(query);
    let finder = (!query.is_empty()).then(|| Finder::new(query.as_bytes()));
    let matches: Vec<CheckOption> = (0..source.row_count())
        .filter_map(|index| source.row_data(index))
        .filter(|value| {
            finder
                .as_ref()
                .is_none_or(|finder| finder.find(fold_search(&value.label).as_bytes()).is_some())
        })
        .map(|mut value| {
            value.checked = checked(&value.key);
            value
        })
        .collect();
    ModelRc::from(matches.as_slice())
}

pub(super) fn source(app: &AppWindow, field: &str) -> ModelRc<CheckOption> {
    match field {
        "Type" => app.get_kind_options(),
        "Continent" => app.get_continent_options(),
        "Subcontinent" => app.get_subcontinent_options(),
        "Region" => app.get_region_options(),
        "Area" => app.get_area_options(),
        "Province" => app.get_province_options(),
        "Topography" => app.get_topography_options(),
        "Vegetation" => app.get_vegetation_options(),
        "Climate" => app.get_climate_options(),
        "Religion" => app.get_religion_options(),
        "Culture" => app.get_culture_options(),
        "Raw material" => app.get_raw_material_options(),
        "Modifier" => app.get_modifier_options(),
        "Coastal" => app.get_coastal_options(),
        "River" => app.get_river_options(),
        "Min river bonus" | "Max river bonus" => app.get_river_level_options(),
        "Harbor suitability" => app.get_harbor_options(),
        "Movement assistance" => app.get_movement_options(),
        _ => ModelRc::default(),
    }
}

fn symbols(
    dataset: &Dataset,
    select: impl Fn(&crate::model::LocationRecord) -> Option<SymbolId>,
) -> HashSet<SymbolId> {
    dataset.stored.locations.iter().filter_map(select).collect()
}

fn model(
    dataset: &Dataset,
    symbols: HashSet<SymbolId>,
    include_missing: bool,
) -> ModelRc<CheckOption> {
    let mut values: Vec<(String, CheckOption)> = symbols
        .into_iter()
        .filter_map(|symbol| {
            let key = dataset.symbol(symbol)?;
            let label = dataset.label(symbol).unwrap_or(key);
            let display = if label == key {
                key.to_owned()
            } else {
                format!("{label}  [{key}]")
            };
            Some((fold_search(&display), option(key, &display)))
        })
        .collect();
    values.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| left.1.label.cmp(&right.1.label))
    });
    let mut options = Vec::with_capacity(values.len().saturating_add(1));
    if include_missing {
        options.push(option("__missing__", "Missing"));
    }
    options.extend(values.into_iter().map(|(_, value)| value));
    ModelRc::from(options.as_slice())
}

fn static_model(values: &[(&str, &str)]) -> ModelRc<CheckOption> {
    let values: Vec<CheckOption> = values
        .iter()
        .map(|(key, label)| option(key, label))
        .collect();
    ModelRc::from(values.as_slice())
}

fn option(key: &str, label: &str) -> CheckOption {
    CheckOption {
        key: SharedString::from(key),
        label: SharedString::from(label),
        checked: false,
    }
}

#[cfg(test)]
mod tests {
    use super::{filtered, static_model};
    use slint::Model;

    #[test]
    fn search_filters_and_marks_checkbox_state() {
        let source = static_model(&[("kourim", "Kouřim"), ("stockholm", "Stockholm")]);
        let result = filtered(source, "kourim", |key| key == "kourim");
        assert_eq!(result.row_count(), 1);
        assert!(result.row_data(0).is_some_and(|value| value.checked));
    }

    #[test]
    fn search_ignores_option_punctuation_and_whitespace() {
        let source = static_model(&[("ras_al_ain", "Ra's al-'Ain"), ("abu_dhabi", "Abu Dhabi")]);
        for query in ["rasalain", "ras al ain", "ras_al_ain", "ras—al—ʿain"] {
            let result = filtered(source.clone(), query, |_| false);
            assert_eq!(result.row_count(), 1, "query did not match: {query:?}");
            assert_eq!(
                result.row_data(0).map(|value| value.key),
                Some("ras_al_ain".into())
            );
        }
    }
}
