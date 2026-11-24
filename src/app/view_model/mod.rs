//! Plain data the views draw: columns, facets, option lists, cell text, detail content and the
//! view-state codec. Nothing here depends on iced widgets except the checklist row trait.

mod column_id;
mod columns;
#[cfg(test)]
mod columns_tests;
mod detail;
mod facet_ui;
mod facets;
#[cfg(test)]
mod facets_meta_tests;
#[cfg(test)]
mod facets_tests;
mod options;
#[cfg(test)]
mod options_tests;
mod ranges;
mod row;
#[cfg(test)]
mod row_tests;
mod selection;
mod sort;
pub(crate) mod view_state;

pub(crate) use column_id::ColumnId;
#[cfg(test)]
pub(crate) use columns::Column;
pub(crate) use columns::{ColumnRows, Columns};
pub(crate) use detail::{DetailField, DetailGroup, LocationDetail, build as build_detail};
pub(crate) use facet_ui::{FacetRows, FacetUi};
pub(crate) use facets::{FacetId, RailSection};
#[cfg(test)]
pub(crate) use options::FacetOption;
pub(crate) use options::{FacetOptions, OptionList, OptionValue};
pub(crate) use ranges::{Bound, RangeField};
#[cfg(test)]
pub(crate) use row::CellTone;
pub(crate) use row::{CellAlign, CellFont, RowContext, tone_color, write_cell};
#[cfg(test)]
pub(crate) use selection::is_selected;
pub(crate) use selection::{active_filter_count, clear, selected_count, set_selected};
pub(crate) use sort::{SortState, column_for_sort, sort_from_token, sort_token};
pub(crate) use view_state::LocationKeys;
