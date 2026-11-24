//! Custom widgets: the virtualized result grid and checklist on a shared windowing core, the
//! pane splitter, and the focus ring that makes stock controls keyboard operable.

pub(crate) mod checklist;
mod focus_ring;
pub(crate) mod result_grid;
mod splitter;
pub(crate) mod window;
#[cfg(test)]
mod window_tests;

pub(crate) use checklist::{Checklist, ChecklistModel};
pub(crate) use focus_ring::{FocusRing, focus_ring};
pub(crate) use result_grid::ResultGrid;
pub(crate) use splitter::Splitter;
