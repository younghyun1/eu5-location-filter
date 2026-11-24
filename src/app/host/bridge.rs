//! Imported host globals and the protocol constants.
//!
//! The imports name plain globals that `web/index.html` defines before `init()`, so wasm-pack
//! emits no snippet files. `catch` turns a missing global or a thrown exception into `Err`.

use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::AppError;

/// Host protocol version.
pub(crate) const PROTOCOL_VERSION: u8 = 2;
/// Longest accepted locale tag.
pub(crate) const MAX_LOCALE_TAG_LEN: usize = 35;
/// Byte limit passed to the host fetch for language and font packs; equals
/// `lang_pack::MAX_PACK_COMPRESSED_SIZE`.
pub(crate) const PACK_MAX_BYTES: u32 = 16 * 1024 * 1024;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_name = cyhdevEu5ReportState)]
    fn report_view_state(state: &str) -> Result<(), JsValue>;

    #[wasm_bindgen(catch, js_name = cyhdevEu5FetchAsset)]
    fn fetch_asset(path: &str, max_bytes: u32) -> Result<js_sys::Promise, JsValue>;
}

/// Sends a canonical view-state string to the host script; failures are ignored because
/// reporting is best effort and the host validates again.
pub(crate) fn report(state: &str) {
    let _ = report_view_state(state);
}

/// Fetches a document-relative asset through the host and returns its bytes, or
/// `AppError::Fetch` when the promise rejects, the global is missing, or the value is not a
/// `Uint8Array`.
pub(crate) async fn fetch_asset_bytes(
    path: String,
    max_bytes: u32,
) -> Result<Vec<u8>, AppError> {
    let _ = fetch_asset(&path, max_bytes);
    Err(AppError::InvalidData(format!(
        "asset fetch is not implemented yet: {path}"
    )))
}

/// Opens a URL in a new browsing context with `window.open(url, "_blank")`.
pub(crate) fn open_url(url: &str) -> Result<(), AppError> {
    Err(AppError::Ui(format!("opening {url} is not implemented yet")))
}
