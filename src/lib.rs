#![deny(
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    clippy::unwrap_used
)]
#![warn(missing_docs)]

//! Import, storage, filtering, and UI support for EU5 map locations.

mod compression;

#[cfg(any(feature = "desktop", all(feature = "web", target_family = "wasm")))]
pub mod app;
#[cfg(feature = "desktop")]
pub mod bundle_diff;
pub mod embedded;
pub mod error;
pub mod filter;
#[cfg(feature = "desktop")]
pub mod font_pack;
#[cfg(feature = "desktop")]
pub mod import;
pub mod index_storage;
pub mod lang_pack;
pub mod model;
#[cfg(feature = "desktop")]
pub mod parser;
#[cfg(feature = "desktop")]
pub mod steam;
pub mod storage;

pub use error::AppError;
pub use lang_pack::GameLanguage;
pub use model::{Dataset, LocationId, LocationRecord, MapColor, StoredDataset, SymbolId};

/// Starts the iced application in the browser canvas.
#[cfg(all(feature = "web", target_family = "wasm"))]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start_web() -> Result<(), wasm_bindgen::JsValue> {
    app::run_web().map_err(|error| wasm_bindgen::JsValue::from_str(&error.to_string()))
}

/// Applies a host palette: `mode` is `"light"` or `"dark"` and `colours` holds the 12 tokens in
/// protocol order as `0xRRGGBB`. Returns `false` and changes nothing on invalid input.
#[cfg(all(feature = "web", target_family = "wasm"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn set_palette(mode: &str, colours: &[u32]) -> bool {
    app::host::set_palette(mode, colours)
}

/// Applies a site locale tag; an unknown tag applies `en-US` and returns `false`.
#[cfg(all(feature = "web", target_family = "wasm"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn set_locale(tag: &str) -> bool {
    app::host::set_locale(tag)
}

/// Applies the site's motion switch; `true` disables palette and drawer animation.
#[cfg(all(feature = "web", target_family = "wasm"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn set_reduced_motion(reduced: bool) {
    app::host::set_reduced_motion(reduced);
}

/// Queues a view-state string for the loaded dataset; returns `false` when the whole string is
/// rejected (empty, too long, outside the charset, or not version `s1`).
#[cfg(all(feature = "web", target_family = "wasm"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn apply_view_state(text: &str) -> bool {
    app::host::apply_view_state(text)
}

/// Applies text typed into the host page's search field, which exists because the browser
/// canvas receives no input-method composition; returns `false` when the text is too long.
#[cfg(all(feature = "web", target_family = "wasm"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn set_search_text(text: &str) -> bool {
    app::host::set_search_text(text)
}

/// Runs bounded real-data filter scans for browser optimization comparisons.
#[cfg(all(feature = "web-benchmark", feature = "web", target_family = "wasm"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn benchmark_filter_work(iterations: u32) -> Result<u32, wasm_bindgen::JsValue> {
    app::host::benchmark_filter_work(iterations)
        .map_err(|error| wasm_bindgen::JsValue::from_str(&error.to_string()))
}
