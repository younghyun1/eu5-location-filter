//! The result grid: one widget owning the column header, the virtualized body and both
//! scrollbars. It borrows result IDs, dataset text, columns, sort and selection, keeps scroll,
//! hover, drag and focus in its own tree state, and draws only visible rows and columns.

mod draw_body;
mod draw_chrome;
mod draw_header;
mod geometry;
mod grid;
mod keys;
mod metrics;
mod paint;
mod pointer;
mod publish;
mod report;
mod state;
#[cfg(test)]
mod tests;
mod touch;
mod widget_impl;

pub(crate) use grid::ResultGrid;
