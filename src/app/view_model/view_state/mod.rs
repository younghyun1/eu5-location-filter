//! The view-state string of the host protocol: escaping, decoding to raw items, canonical
//! encoding, and resolution against the loaded dataset's option lists. Target-independent so
//! its tests run natively.

mod decode;
mod encode;
mod limits;
mod resolve;
#[cfg(test)]
mod tests;
mod text;

pub(crate) use decode::{RawItem, RawViewState, decode, passes_whole_checks};
pub(crate) use encode::{ViewSnapshot, encode_into};
pub(crate) use limits::{
    MAX_KEY_BYTES, MAX_NUMBER_CHARS, MAX_SEARCH_BYTES, VIEW_STATE_MAX_FIELDS, VIEW_STATE_MAX_ITEMS,
    VIEW_STATE_MAX_LEN, VIEW_STATE_VERSION, facet_field,
};
pub(crate) use resolve::{LocationKeys, ResolvedView, resolve};
pub(crate) use text::{escape_into, unescape};
