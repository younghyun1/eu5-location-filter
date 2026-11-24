//! Filter rail: pinned header, collapsed strip, and the scrolling sections of facet groups,
//! toggles, the exact-colour field and the numeric ranges.

mod facet_group;
mod fields;
mod rail;

pub(crate) use rail::{RailModel, view};
