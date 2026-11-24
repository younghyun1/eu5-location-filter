//! Bounded decoding of a view-state string into raw, dataset-independent values.

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
    let _ = text;
    false
}

/// Decodes a string. Fails as a whole only when `passes_whole_checks` fails; otherwise bad
/// fields and items are dropped, unknown and repeated fields are ignored, and at most
/// `VIEW_STATE_MAX_FIELDS` fields and `VIEW_STATE_MAX_ITEMS` items per list are read.
pub(crate) fn decode(text: &str) -> Result<RawViewState, AppError> {
    let _ = text;
    Err(AppError::InvalidData(
        "view-state decoding is not implemented yet".to_owned(),
    ))
}
