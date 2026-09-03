//! Dataset-backed dropdown options for categorical facets.

use std::collections::HashSet;

use memchr::memmem::Finder;
use slint::{Model, ModelRc, SharedString};

use super::AppWindow;
use crate::filter::fold_search;
use crate::model::{Dataset, SymbolId};

pub(super) fn install(app: &AppWindow, dataset: &Dataset) {
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
    let river_levels: Vec<SharedString> = std::iter::once(SharedString::from("Any"))
        .chain((0..=dataset.stored.river_widths.level_count).map(|level| level.to_string().into()))
        .collect();
    app.set_river_level_options(ModelRc::from(river_levels.as_slice()));
}

pub(super) fn filtered(app: &AppWindow, field: &str, query: &str) -> ModelRc<SharedString> {
    let source = match field {
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
        "Min river level" | "Max river level" => app.get_river_level_options(),
        "Kind" => static_model(&["Any", "Land", "Sea", "Lake", "Impassable", "Unknown"]),
        "Coastal" => static_model(&["Any", "Yes", "No"]),
        "River" | "Harbor suitability" | "Movement assistance" => {
            static_model(&["Any", "Present", "Missing"])
        }
        _ => ModelRc::default(),
    };
    let query = fold_search(query);
    filter_source(source, &query)
}

fn filter_source(source: ModelRc<SharedString>, query: &str) -> ModelRc<SharedString> {
    if query.is_empty() {
        return source;
    }
    let finder = Finder::new(query.as_bytes());
    let matches: Vec<SharedString> = (0..source.row_count())
        .filter_map(|index| source.row_data(index))
        .filter(|value| finder.find(fold_search(value).as_bytes()).is_some())
        .collect();
    ModelRc::from(matches.as_slice())
}

fn static_model(values: &[&str]) -> ModelRc<SharedString> {
    let values: Vec<SharedString> = values.iter().map(|value| (*value).into()).collect();
    ModelRc::from(values.as_slice())
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
) -> ModelRc<SharedString> {
    let mut values: Vec<(String, String)> = symbols
        .into_iter()
        .filter_map(|symbol| {
            let key = dataset.symbol(symbol)?;
            let label = dataset.label(symbol).unwrap_or(key);
            let display = if label == key {
                key.to_owned()
            } else {
                format!("{label}  [{key}]")
            };
            Some((fold_search(&display), display))
        })
        .collect();
    values.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    let mut options = Vec::with_capacity(values.len().saturating_add(2));
    options.push(SharedString::from("Any"));
    if include_missing {
        options.push(SharedString::from("Missing"));
    }
    options.extend(values.into_iter().map(|(_, display)| display.into()));
    ModelRc::from(options.as_slice())
}

#[cfg(test)]
mod tests;
