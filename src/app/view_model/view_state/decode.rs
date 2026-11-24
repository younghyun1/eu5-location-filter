//! Bounded decoding of a view-state string into raw, dataset-independent values.

use super::fields::{Field, item_rule};
use super::limits::{VIEW_STATE_MAX_FIELDS, VIEW_STATE_MAX_LEN, VIEW_STATE_VERSION};
use super::parse;
use super::text::is_raw;
use crate::AppError;
use crate::app::view_model::{FacetId, SortState};
use crate::filter::FloatRange;
use crate::model::MapColor;

/// One decoded list item before it is resolved against a facet's options.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RawItem {
    /// A lone `!`: the missing value.
    Missing,
    /// Decoded key text: symbol identifier, kind word, presence word or tier digits.
    Key(String),
}

/// Everything a string can carry, with invalid fields already reset to their defaults.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RawViewState {
    /// `q`: search text, empty by default.
    pub(crate) search: String,
    /// Facet lists indexed by `FacetId::index`. `None` means the field was absent, which for
    /// Type means land only; `Some(vec![])` for Type means every kind.
    pub(crate) facets: [Option<Vec<RawItem>>; FacetId::COUNT],
    /// `fo`: food-producing raw materials only.
    pub(crate) food_only: bool,
    /// `c`: exact map colour.
    pub(crate) rgb: Option<MapColor>,
    /// `hr`: harbor suitability bounds.
    pub(crate) harbor_range: FloatRange,
    /// `mx`: first movement component bounds.
    pub(crate) movement_x: FloatRange,
    /// `my`: second movement component bounds.
    pub(crate) movement_y: FloatRange,
    /// `si`: impassable locations shown, `true` by default.
    pub(crate) show_impassable: bool,
    /// `s`: sort, name ascending by default.
    pub(crate) sort: SortState,
    /// `l`: selected location key.
    pub(crate) selected: Option<String>,
}

impl Default for RawViewState {
    fn default() -> Self {
        Self {
            search: String::new(),
            facets: Default::default(),
            food_only: false,
            rgb: None,
            harbor_range: FloatRange::default(),
            movement_x: FloatRange::default(),
            movement_y: FloatRange::default(),
            show_impassable: true,
            sort: SortState::default(),
            selected: None,
        }
    }
}

/// Whole-string checks shared with the browser export: non-empty, at most
/// `VIEW_STATE_MAX_LEN` bytes, only `A-Z a-z 0-9 _ . ~ * ! -`, and the first token is `s1`.
#[must_use]
pub(crate) fn passes_whole_checks(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= VIEW_STATE_MAX_LEN
        && text
            .bytes()
            .all(|byte| is_raw(byte) || matches!(byte, b'~' | b'*' | b'!'))
        && text.split('~').next() == Some(VIEW_STATE_VERSION)
}

/// Decodes a string. Fails as a whole only when `passes_whole_checks` fails; otherwise bad
/// fields and items are dropped, unknown and repeated fields are ignored, and at most
/// `VIEW_STATE_MAX_FIELDS` fields and `VIEW_STATE_MAX_ITEMS` items per list are read.
pub(crate) fn decode(text: &str) -> Result<RawViewState, AppError> {
    if !passes_whole_checks(text) {
        return Err(AppError::InvalidData(
            "view-state string is empty, too long, outside the charset or not version s1"
                .to_owned(),
        ));
    }
    let mut state = RawViewState::default();
    let mut seen = 0_u32;
    for token in text.split('~').skip(1).take(VIEW_STATE_MAX_FIELDS) {
        let (name, value) = match token.split_once('.') {
            Some(parts) => parts,
            None => continue,
        };
        let field = match Field::from_name(name) {
            Some(field) => field,
            None => continue,
        };
        // The first occurrence wins even when its value is invalid.
        let bit = field.bit();
        if seen & bit != 0 {
            continue;
        }
        seen |= bit;
        apply_field(&mut state, field, value);
    }
    Ok(state)
}

/// Stores one field's value; an invalid value leaves the default in place.
fn apply_field(state: &mut RawViewState, field: Field, value: &str) {
    match field {
        Field::Search => {
            if let Some(search) = parse::search(value) {
                state.search = search;
            }
        }
        Field::Facet(facet) => {
            if let Some(slot) = state.facets.get_mut(facet.index()) {
                *slot = parse::list(value, item_rule(facet));
            }
        }
        Field::FoodOnly => {
            if let Some(food_only) = parse::flag(value) {
                state.food_only = food_only;
            }
        }
        Field::Rgb => state.rgb = parse::rgb(value),
        Field::HarborRange => state.harbor_range = parse::range(value).unwrap_or_default(),
        Field::MovementX => state.movement_x = parse::range(value).unwrap_or_default(),
        Field::MovementY => state.movement_y = parse::range(value).unwrap_or_default(),
        Field::ShowImpassable => {
            if let Some(shown) = parse::flag(value) {
                state.show_impassable = shown;
            }
        }
        Field::Sort => {
            if let Some(sort) = parse::sort(value) {
                state.sort = sort;
            }
        }
        Field::Selected => state.selected = parse::key(value),
    }
}
