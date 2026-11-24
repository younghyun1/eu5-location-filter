//! The result grid: one widget owning the column header, the virtualized body and both
//! scrollbars. It borrows result IDs, dataset text, columns, sort and selection, keeps scroll,
//! hover, drag and focus in its own tree state, and draws only visible rows and columns.

mod grid;
mod state;
mod widget_impl;

pub(crate) use grid::ResultGrid;
pub(crate) use state::{Axis, GridDrag, GridState, TouchPan};
