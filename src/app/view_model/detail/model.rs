//! Detail pane data types.

use crate::app::i18n::Text;
use crate::app::view_model::row::CellTone;
use crate::model::{LocationId, MapColor};

/// One label and value row.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DetailField {
    /// Field label.
    pub(crate) label: Text,
    /// Formatted value, or `value.none`.
    pub(crate) value: String,
    /// Value colour class.
    pub(crate) tone: CellTone,
    /// Whether the value is technical text laid out left to right in Mono.
    pub(crate) mono: bool,
}

/// One titled group of fields.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DetailGroup {
    /// Group title.
    pub(crate) title: Text,
    /// Rows in contract order.
    pub(crate) fields: Vec<DetailField>,
    /// Note below the group, for the capacity breakdown.
    pub(crate) note: Option<Text>,
}

/// Everything the detail pane shows for a location.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LocationDetail {
    /// Location shown.
    pub(crate) id: LocationId,
    /// Map colour for the colour block.
    pub(crate) color: MapColor,
    /// `#RRGGBB`.
    pub(crate) hex: String,
    /// `format.rgb` text.
    pub(crate) rgb: String,
    /// Localized name with English fallback.
    pub(crate) name: String,
    /// Symbol key.
    pub(crate) key: String,
    /// Continent, subcontinent, region, area and province labels; the pane joins them with the
    /// direction's separator glyph.
    pub(crate) breadcrumb: Vec<String>,
    /// Classification, society, economy, coast and movement, river, capacity.
    pub(crate) groups: Vec<DetailGroup>,
}

