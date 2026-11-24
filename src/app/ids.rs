//! Widget identifiers shared by views, focus handling, scroll operations and simulator tests.

use iced::widget::Id;

use super::view_model::{Bound, FacetId, RangeField};

/// Global search field in the header.
pub(crate) const SEARCH: Id = Id::new("header.search");
/// Desktop Language menu opener.
pub(crate) const LANGUAGE_BUTTON: Id = Id::new("header.language");
/// Desktop Appearance menu opener.
pub(crate) const APPEARANCE_BUTTON: Id = Id::new("header.appearance");
/// About dialog opener.
pub(crate) const ABOUT_BUTTON: Id = Id::new("header.about");
/// Narrow-layout filter drawer opener.
pub(crate) const FILTERS_BUTTON: Id = Id::new("header.filters");
/// Narrow-layout detail drawer opener.
pub(crate) const DETAILS_BUTTON: Id = Id::new("header.details");
/// Rail collapse and expand button.
pub(crate) const RAIL_COLLAPSE: Id = Id::new("rail.collapse");
/// Rail Clear all button.
pub(crate) const RAIL_CLEAR_ALL: Id = Id::new("rail.clear_all");
/// Scrollable rail content.
pub(crate) const RAIL_SCROLL: Id = Id::new("rail.scroll");
/// Splitter between the rail and the results.
pub(crate) const RAIL_SPLITTER: Id = Id::new("rail.splitter");
/// Exact-colour field.
pub(crate) const RGB_FIELD: Id = Id::new("rail.rgb");
/// Columns button in the results toolbar.
pub(crate) const COLUMNS_BUTTON: Id = Id::new("results.columns");
/// Show details button in the results toolbar.
pub(crate) const SHOW_DETAILS_BUTTON: Id = Id::new("results.show_details");
/// Result grid.
pub(crate) const GRID: Id = Id::new("results.grid");
/// Splitter between the results and the detail pane.
pub(crate) const DETAIL_SPLITTER: Id = Id::new("detail.splitter");
/// Detail pane hide button.
pub(crate) const DETAIL_HIDE: Id = Id::new("detail.hide");
/// Scrollable detail content.
pub(crate) const DETAIL_SCROLL: Id = Id::new("detail.scroll");
/// Column menu checklist.
pub(crate) const COLUMN_MENU_LIST: Id = Id::new("column_menu.list");
/// Scrollable About dialog content.
pub(crate) const ABOUT_SCROLL: Id = Id::new("about.scroll");
/// Desktop installation directory field on the load-error card.
pub(crate) const INSTALL_PATH: Id = Id::new("error.install_path");

/// Checklist of one facet group.
#[must_use]
pub(crate) fn facet_list(facet: FacetId) -> Id {
    Id::from(format!("facet.{}.list", facet.key()))
}

/// Option search field of one facet group.
#[must_use]
pub(crate) fn facet_search(facet: FacetId) -> Id {
    Id::from(format!("facet.{}.search", facet.key()))
}

/// Fold button of one facet group.
#[must_use]
pub(crate) fn facet_fold(facet: FacetId) -> Id {
    Id::from(format!("facet.{}.fold", facet.key()))
}

/// One bound field of a numeric range.
#[must_use]
pub(crate) fn range_field(field: RangeField, bound: Bound) -> Id {
    Id::from(format!("range.{}.{}", field.key(), bound.key()))
}
