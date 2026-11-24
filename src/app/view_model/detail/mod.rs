//! Detail pane content for one location, built once per selection, locale or pack change.

mod build;
mod groups;
mod model;

pub(crate) use build::build;
pub(crate) use model::{DetailField, DetailGroup, LocationDetail};
