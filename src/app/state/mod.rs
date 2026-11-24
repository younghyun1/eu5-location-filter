//! Application state: one `App` value owns everything the view reads.

mod app;
mod appearance;
mod fields;
mod overlay;
mod packs;
mod phase;

pub(crate) use app::{App, LaunchPaths};
pub(crate) use appearance::Appearance;
pub(crate) use fields::{FieldBuffer, TextFields};
pub(crate) use overlay::{
    FocusRegion, FocusState, Notice, OverlayState, Surface, drawer_animation,
};
pub(crate) use packs::{PackFailure, PackState};
pub(crate) use phase::{LoadFailure, LoadProgress, Phase};
