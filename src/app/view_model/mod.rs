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
pub(crate) use columns::{Column, ColumnRows, Columns};
pub(crate) use detail::{DetailField, DetailGroup, LocationDetail, build as build_detail};
pub(crate) use facet_ui::{FacetRows, FacetUi};
pub(crate) use facets::{FacetId, RailSection};
pub(crate) use options::{FacetOption, FacetOptions, OptionList, OptionValue};
pub(crate) use ranges::{Bound, RangeField};
pub(crate) use row::{CellAlign, CellFont, CellTone, RowContext, tone_color, write_cell};
pub(crate) use selection::{active_filter_count, clear, is_selected, selected_count, set_selected};
pub(crate) use sort::{SortState, column_for_sort, sort_from_token, sort_token};
pub(crate) use view_state::LocationKeys;
