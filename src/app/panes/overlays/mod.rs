//! Overlays above the body: loading and error cards over veils, the notice bar, drawers with
//! their scrim, and the modal frame for the About dialog.

mod drawer;
mod error;
mod loading;
mod notice;

pub(crate) use drawer::{drawer, modal, scrim};
pub(crate) use error::error_card;
pub(crate) use loading::loading_card;
pub(crate) use notice::notice_bar;
