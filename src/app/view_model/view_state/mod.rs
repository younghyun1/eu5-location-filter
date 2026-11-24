//! The view-state string of the host protocol: escaping, decoding to raw items, canonical
//! encoding, and resolution against the loaded dataset's option lists. Target-independent so
//! its tests run natively.

mod decode;
mod encode;
mod fields;
mod limits;
mod parse;
mod resolve;
#[cfg(test)]
mod tests;
mod text;

pub(crate) use decode::decode;
#[cfg(any(test, all(feature = "web", target_family = "wasm")))]
pub(crate) use decode::passes_whole_checks;
#[cfg(test)]
pub(crate) use decode::{RawItem, RawViewState};
pub(crate) use encode::{ViewSnapshot, encode_into};
pub(crate) use limits::MAX_SEARCH_BYTES;
#[cfg(test)]
pub(crate) use limits::VIEW_STATE_MAX_LEN;
#[cfg(test)]
pub(crate) use resolve::ResolvedView;
pub(crate) use resolve::{LocationKeys, resolve};
#[cfg(test)]
pub(crate) use text::{escape_into, unescape};
