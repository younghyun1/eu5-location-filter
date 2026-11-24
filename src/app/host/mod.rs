//! Browser host bridge (wasm only): export targets that validate host input into fixed
//! latest-wins slots, the subscription that drains them, the imported reporting and fetch
//! globals, and the benchmark export.

#[cfg(feature = "web-benchmark")]
mod benchmark;
mod bridge;
mod exports;
mod inbox;

#[cfg(feature = "web-benchmark")]
pub(crate) use benchmark::benchmark_filter_work;
pub(crate) use bridge::{
    MAX_LOCALE_TAG_LEN, PACK_MAX_BYTES, PROTOCOL_VERSION, fetch_asset_bytes, open_url, report,
};
pub(crate) use exports::{apply_view_state, set_locale, set_palette, set_reduced_motion};
pub(crate) use inbox::{subscription, take_snapshot};
