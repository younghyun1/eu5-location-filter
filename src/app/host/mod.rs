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
    PACK_MAX_BYTES, SearchOverlay, fetch_asset_bytes, focus_search_overlay, open_url, report,
    sync_search_overlay,
};
pub(crate) use exports::{
    apply_view_state, set_locale, set_palette, set_reduced_motion, set_search_text,
};
pub(crate) use inbox::{subscription, take_snapshot};
