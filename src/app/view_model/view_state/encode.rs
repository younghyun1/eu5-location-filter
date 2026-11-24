//! Canonical encoding of the current view: table order, defaults omitted, sorted and
//! deduplicated items, shortest round-trip numbers, uppercase hex.

use std::collections::HashSet;
use std::fmt::Write as _;

use super::fields::{Field, PROTOCOL_ORDER, coastal_word, kind_word, presence_word};
use super::limits::{MAX_KEY_BYTES, MAX_SEARCH_BYTES, VIEW_STATE_MAX_LEN, VIEW_STATE_VERSION};
use super::parse;
use super::text::{escape_into, has_control};
use crate::app::view_model::{FacetId, SortState, sort_token};
use crate::filter::{FilterSet, FloatRange};
use crate::model::{Dataset, LocationKind, SymbolId};

/// The parts of the application state a view-state string records.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ViewSnapshot<'a> {
    /// Active filters, including the search.
    pub(crate) filters: &'a FilterSet,
    /// Active sort.
    pub(crate) sort: SortState,
    /// Key of the selected location.
    pub(crate) selected_key: Option<&'a str>,
    /// Dataset, for symbol keys of selected facet values.
    pub(crate) dataset: &'a Dataset,
}

/// Clears `out` and writes the canonical string. Fields or items the decoder would reject are
/// omitted; when the result would exceed `VIEW_STATE_MAX_LEN` the default `s1` is written.
pub(crate) fn encode_into(out: &mut String, view: &ViewSnapshot<'_>) {
    out.clear();
    out.push_str(VIEW_STATE_VERSION);
    // One scratch list reused by every list field; it only allocates for selected items.
    let mut items: Vec<String> = Vec::new();
    for field in PROTOCOL_ORDER {
        write_field(out, &mut items, field, view);
        if out.len() > VIEW_STATE_MAX_LEN {
            break;
        }
    }
    if out.len() > VIEW_STATE_MAX_LEN {
        out.clear();
        out.push_str(VIEW_STATE_VERSION);
    }
}

/// Appends one field when it differs from its default and has an encodable value.
fn write_field(out: &mut String, items: &mut Vec<String>, field: Field, view: &ViewSnapshot<'_>) {
    let filters = view.filters;
    match field {
        Field::Search => {
            let search = filters.search.as_str();
            if !search.is_empty() && search.len() <= MAX_SEARCH_BYTES && !has_control(search) {
                start_field(out, field);
                escape_into(out, search);
            }
        }
        Field::Facet(facet) => write_facet(out, items, facet, view),
        Field::FoodOnly => {
            if filters.food_producing_only {
                start_field(out, field);
                out.push('1');
            }
        }
        Field::Rgb => {
            if let Some(color) = filters.rgb
                && color.0 <= 0x00FF_FFFF
            {
                start_field(out, field);
                // Writing into a String cannot fail.
                let _ = write!(out, "{:06X}", color.0);
            }
        }
        Field::HarborRange => write_range(out, field, filters.harbor_range),
        Field::MovementX => write_range(out, field, filters.movement_x),
        Field::MovementY => write_range(out, field, filters.movement_y),
        Field::ShowImpassable => {
            if !filters.show_impassable {
                start_field(out, field);
                out.push('0');
            }
        }
        Field::Sort => {
            if view.sort != SortState::default() {
                start_field(out, field);
                out.push_str(sort_token(view.sort.field));
                out.push_str(if view.sort.ascending { "*a" } else { "*d" });
            }
        }
        Field::Selected => {
            if let Some(key) = view.selected_key
                && !key.is_empty()
                && key.len() <= MAX_KEY_BYTES
            {
                start_field(out, field);
                escape_into(out, key);
            }
        }
    }
}

/// Writes `~name.`.
fn start_field(out: &mut String, field: Field) {
    out.push('~');
    out.push_str(field.name());
    out.push('.');
}

/// Appends a facet list. Type is omitted only when it is exactly land; its empty set is written
/// as `k.` (every kind). Other facets are omitted when empty.
fn write_facet(out: &mut String, items: &mut Vec<String>, facet: FacetId, view: &ViewSnapshot<'_>) {
    let filters = view.filters;
    if facet == FacetId::Kind {
        if filters.kinds.len() == 1 && filters.kinds.contains(&LocationKind::Land) {
            return;
        }
        items.clear();
        items.extend(filters.kinds.iter().map(|kind| kind_word(*kind).to_owned()));
        write_items(out, items, facet, true);
        return;
    }
    items.clear();
    let dataset = view.dataset;
    match facet {
        FacetId::Kind => {}
        FacetId::Topography => push_symbols(items, dataset, &filters.topographies),
        FacetId::Vegetation => push_optional(items, dataset, &filters.vegetation, true),
        FacetId::Climate => push_optional(items, dataset, &filters.climates, true),
        FacetId::Continent => push_optional(items, dataset, &filters.continents, false),
        FacetId::Subcontinent => push_optional(items, dataset, &filters.subcontinents, false),
        FacetId::Region => push_optional(items, dataset, &filters.regions, false),
        FacetId::Area => push_optional(items, dataset, &filters.areas, false),
        FacetId::Province => push_optional(items, dataset, &filters.provinces, false),
        FacetId::Religion => push_optional(items, dataset, &filters.religions, true),
        FacetId::Culture => push_optional(items, dataset, &filters.cultures, true),
        FacetId::RawMaterial => push_optional(items, dataset, &filters.raw_materials, true),
        FacetId::Modifier => push_optional(items, dataset, &filters.modifiers, true),
        FacetId::Coastal => {
            items.extend(filters.coastal.iter().map(|value| coastal_word(*value).to_owned()));
        }
        FacetId::RiverPresence => push_presence(items, &filters.river_presence),
        FacetId::HarborPresence => push_presence(items, &filters.harbor_presence),
        FacetId::MovementPresence => push_presence(items, &filters.movement_presence),
        FacetId::RiverLevelMin => push_tiers(items, &filters.river_level_min),
        FacetId::RiverLevelMax => push_tiers(items, &filters.river_level_max),
    }
    write_items(out, items, facet, false);
}

/// Sorts and deduplicates encoded items and writes them; an empty list is written only when
/// `write_empty` is set.
fn write_items(out: &mut String, items: &mut Vec<String>, facet: FacetId, write_empty: bool) {
    if items.is_empty() && !write_empty {
        return;
    }
    items.sort_unstable();
    items.dedup();
    start_field(out, Field::Facet(facet));
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            out.push('*');
        }
        out.push_str(item);
    }
}

/// Escaped key of a symbol, or `None` when it is missing, empty or longer than the decoder
/// accepts.
fn symbol_item(dataset: &Dataset, id: SymbolId) -> Option<String> {
    let key = dataset.symbol(id)?;
    if key.is_empty() || key.len() > MAX_KEY_BYTES {
        return None;
    }
    let mut item = String::with_capacity(key.len());
    escape_into(&mut item, key);
    Some(item)
}

/// Items of a facet without a missing value.
fn push_symbols(items: &mut Vec<String>, dataset: &Dataset, set: &HashSet<SymbolId>) {
    items.extend(set.iter().filter_map(|id| symbol_item(dataset, *id)));
}

/// Items of a facet whose set can hold `None`; `None` is written as `!` only where the facet
/// has a missing option, and omitted otherwise because the decoder would drop it.
fn push_optional(
    items: &mut Vec<String>,
    dataset: &Dataset,
    set: &HashSet<Option<SymbolId>>,
    missing: bool,
) {
    for value in set {
        match value {
            Some(id) => items.extend(symbol_item(dataset, *id)),
            None if missing => items.push("!".to_owned()),
            None => {}
        }
    }
}

/// Presence words.
fn push_presence(items: &mut Vec<String>, set: &HashSet<bool>) {
    items.extend(set.iter().map(|value| presence_word(*value).to_owned()));
}

/// Tier digits; zero is omitted because the decoder accepts only 1 to 255.
fn push_tiers(items: &mut Vec<String>, set: &HashSet<u8>) {
    items.extend(set.iter().filter(|tier| **tier >= 1).map(u8::to_string));
}

/// Appends `~name.min*max` when at least one bound is encodable.
fn write_range(out: &mut String, field: Field, range: FloatRange) {
    let min = range.min.and_then(number_text);
    let max = range.max.and_then(number_text);
    if min.is_none() && max.is_none() {
        return;
    }
    start_field(out, field);
    if let Some(min) = min {
        out.push_str(&min);
    }
    out.push('*');
    if let Some(max) = max {
        out.push_str(&max);
    }
}

/// Shortest decimal that parses back to the same value, as `f32` `Display` writes it (no
/// exponent), with negative zero as `0`. `None` when the decoder would not read it back, which
/// covers non-finite values and the length limit.
fn number_text(value: f32) -> Option<String> {
    if !value.is_finite() {
        return None;
    }
    let mut text = String::new();
    // Adding positive zero turns -0.0 into 0.0; writing into a String cannot fail.
    let _ = write!(text, "{}", value + 0.0);
    match parse::number(&text) {
        Some(parsed) if parsed.to_bits() == (value + 0.0).to_bits() => Some(text),
        Some(_) | None => None,
    }
}
