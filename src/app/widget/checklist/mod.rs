//! Virtualized checkbox list on the shared windowing core, used for facet options (thousands of
//! provinces, areas and cultures) and the column menu. Focusable: arrows, page keys, Home and
//! End move the active option and Space toggles it.

mod draw;
mod geometry;
mod input;
mod list;
mod model;
mod state;
#[cfg(test)]
mod tests;
mod widget_impl;

pub(crate) use list::{Checklist, OnToggle};
pub(crate) use model::ChecklistModel;
pub(crate) use state::ChecklistState;
