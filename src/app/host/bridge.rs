//! Imported host globals and the protocol constants.
//!
//! The imports name plain globals that `web/index.html` defines before `init()`, so wasm-pack
//! emits no snippet files. `catch` turns a missing global or a thrown exception into `Err`.

use std::cell::RefCell;

use js_sys::Uint8Array;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

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

    #[wasm_bindgen(catch, js_name = cyhdevEu5SearchOverlay)]
    fn search_overlay(
        visible: bool,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        text: &str,
        placeholder: &str,
        rtl: bool,
    ) -> Result<(), JsValue>;

    #[wasm_bindgen(catch, js_name = cyhdevEu5FocusSearch)]
    fn focus_search() -> Result<(), JsValue>;
}

/// Placement and content of the host page's search field, in CSS pixels of the frame
/// document (equal to iced logical pixels, because the canvas fills the document).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SearchOverlay<'a> {
    /// Bounds of the header search slot.
    pub(crate) bounds: iced::Rectangle,
    /// Current `FilterSet::search`; the host leaves its field alone while composing.
    pub(crate) text: &'a str,
    /// Localized placeholder.
    pub(crate) placeholder: &'a str,
    /// Right-to-left interface direction.
    pub(crate) rtl: bool,
}

/// Last values sent to `cyhdevEu5SearchOverlay`. Update runs after every message, so without this
/// every pointer move would cross into JavaScript. One entry; its strings are reused in place.
#[derive(Debug)]
struct SentOverlay {
    /// Whether the field was shown.
    visible: bool,
    /// Bounds sent with the last visible call.
    bounds: iced::Rectangle,
    /// Text sent with the last visible call.
    text: String,
    /// Placeholder sent with the last visible call.
    placeholder: String,
    /// Direction sent with the last visible call.
    rtl: bool,
}

impl SentOverlay {
    /// Whether `overlay` would send exactly what was sent last.
    fn matches(&self, overlay: Option<&SearchOverlay<'_>>) -> bool {
        match overlay {
            None => !self.visible,
            Some(overlay) => {
                self.visible
                    && self.bounds == overlay.bounds
                    && self.rtl == overlay.rtl
                    && self.text == overlay.text
                    && self.placeholder == overlay.placeholder
            }
        }
    }

    /// Records a successful call.
    fn record(&mut self, overlay: Option<&SearchOverlay<'_>>) {
        match overlay {
            None => self.visible = false,
            Some(overlay) => {
                self.visible = true;
                self.bounds = overlay.bounds;
                self.rtl = overlay.rtl;
                overlay.text.clone_into(&mut self.text);
                overlay.placeholder.clone_into(&mut self.placeholder);
            }
        }
    }
}

thread_local! {
    // The host page starts with the field hidden, so the initial value suppresses a first hide.
    static SENT_OVERLAY: RefCell<SentOverlay> = const {
        RefCell::new(SentOverlay {
            visible: false,
            bounds: iced::Rectangle {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
            text: String::new(),
            placeholder: String::new(),
            rtl: false,
        })
    };
}

/// Shows and positions the host search field, or hides it for `None` (dataset not ready, a
/// modal surface open, or the slot not laid out yet). Failures are ignored: without the host
/// function the browser build simply has no input-method search.
pub(crate) fn sync_search_overlay(overlay: Option<SearchOverlay<'_>>) {
    SENT_OVERLAY.with(|cell| {
        // A failed borrow means a call is already in progress; skipping it is harmless because
        // the next update sends the current values again.
        let Ok(mut sent) = cell.try_borrow_mut() else {
            return;
        };
        if sent.matches(overlay.as_ref()) {
            return;
        }
        let result = match overlay.as_ref() {
            Some(shown) => search_overlay(
                true,
                f64::from(shown.bounds.x),
                f64::from(shown.bounds.y),
                f64::from(shown.bounds.width),
                f64::from(shown.bounds.height),
                shown.text,
                shown.placeholder,
                shown.rtl,
            ),
            None => search_overlay(false, 0.0, 0.0, 0.0, 0.0, "", "", false),
        };
        // Only a delivered call is recorded, so a transient failure is retried on the next update.
        if result.is_ok() {
            sent.record(overlay.as_ref());
        }
    });
}

/// Moves keyboard focus to the host search field and selects its text (Ctrl+F in the browser).
pub(crate) fn focus_search_overlay() {
    // Best effort: a missing global leaves focus where it is.
    let _ = focus_search();
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
    let promise = match fetch_asset(&path, max_bytes) {
        Ok(promise) => promise,
        Err(error) => return Err(fetch_error(&path, "the host fetch could not start", &error)),
    };
    // Default type parameters keep this form valid for both the one-parameter JsFuture of
    // js-sys 0.3.104 and the two-parameter one of 0.3.107.
    let value = match JsFuture::from(promise).await {
        Ok(value) => value,
        Err(error) => return Err(fetch_error(&path, "the host fetch failed", &error)),
    };
    let bytes = match value.dyn_into::<Uint8Array>() {
        Ok(bytes) => bytes,
        Err(_) => {
            return Err(AppError::Fetch(format!(
                "{path}: the host fetch did not resolve to a Uint8Array"
            )));
        }
    };
    // The host enforces the limit too; checking here keeps an oversized copy out of wasm memory.
    let length = bytes.length();
    if length > max_bytes {
        return Err(AppError::Fetch(format!(
            "{path}: {length} bytes exceed the {max_bytes} byte limit"
        )));
    }
    Ok(bytes.to_vec())
}

/// Fetch failure with the JavaScript error's text.
fn fetch_error(path: &str, context: &str, error: &JsValue) -> AppError {
    let detail = match error.as_string() {
        Some(text) => text,
        None => format!("{error:?}"),
    };
    AppError::Fetch(format!("{path}: {context}: {detail}"))
}

/// Opens a URL in a new browsing context with `window.open(url, "_blank")`. The `noopener`
/// feature keeps the new page from reaching back into the frame, which also makes `open`
/// return `null` on success, so only a thrown exception counts as failure.
pub(crate) fn open_url(url: &str) -> Result<(), AppError> {
    if !url.starts_with("https://") {
        return Err(AppError::Ui(format!("refusing to open a non-HTTPS link: {url}")));
    }
    let window = match web_sys::window() {
        Some(window) => window,
        None => return Err(AppError::Ui(format!("no browser window to open {url}"))),
    };
    match window.open_with_url_and_target_and_features(url, "_blank", "noopener,noreferrer") {
        Ok(_) => Ok(()),
        Err(error) => Err(AppError::Ui(format!("opening {url} failed: {error:?}"))),
    }
}
