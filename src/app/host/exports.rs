//! Bodies of the `#[wasm_bindgen]` exports declared in `lib.rs`. Each validates its input
//! again (the host script already did), writes a slot, and never calls back into JavaScript.

use super::inbox::store;
use crate::app::i18n::Locale;
use crate::app::theme::Palette;
use crate::app::view_model::view_state::{MAX_SEARCH_BYTES, passes_whole_checks};

/// Stores a palette when `Palette::from_host` accepts it; always marks the host connected.
pub(crate) fn set_palette(mode: &str, colours: &[u32]) -> bool {
    let palette = Palette::from_host(mode, colours);
    let accepted = palette.is_some();
    let stored = store(|inbox| {
        inbox.connected = true;
        if palette.is_some() {
            inbox.palette = palette;
        }
    });
    accepted && stored
}

/// Stores the locale for an exact tag, or `en-US` and `false` for any other tag.
pub(crate) fn set_locale(tag: &str) -> bool {
    let locale = Locale::from_tag(tag);
    let stored = store(|inbox| inbox.locale = Some(locale.unwrap_or(Locale::EnUs)));
    locale.is_some() && stored
}

/// Stores the reduced-motion flag.
pub(crate) fn set_reduced_motion(reduced: bool) {
    let _ = store(|inbox| inbox.reduced_motion = Some(reduced));
}

/// Stores a view-state string that passes the whole-string checks.
pub(crate) fn apply_view_state(text: &str) -> bool {
    if !passes_whole_checks(text) {
        return false;
    }
    store(|inbox| inbox.view_state = Some(text.to_owned()))
}

/// Stores the host search field's text; rejects text longer than `MAX_SEARCH_BYTES`.
pub(crate) fn set_search_text(text: &str) -> bool {
    if text.len() > MAX_SEARCH_BYTES {
        return false;
    }
    store(|inbox| inbox.search_text = Some(text.to_owned()))
}
