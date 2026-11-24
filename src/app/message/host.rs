//! Values drained from the browser host inbox.

use crate::app::i18n::Locale;
use crate::app::theme::Palette;

/// Host values drained from the browser inbox in one wake.
#[derive(Clone, Debug)]
pub(crate) enum HostMessage {
    /// Latest value of every slot that changed since the previous drain.
    #[cfg_attr(
        not(all(feature = "web", target_family = "wasm")),
        allow(dead_code, reason = "only the browser host inbox drains snapshots")
    )]
    Update(HostSnapshot),
    /// Window-space bounds of the header search slot, published by the `Anchor` widget so the
    /// host page can place its real text field (which supports input methods) over it.
    SearchAnchor(iced::Rectangle),
}

/// One drain of the browser inbox; each slot holds only the latest value.
#[derive(Clone, Debug, Default)]
#[cfg_attr(
    not(all(feature = "web", target_family = "wasm")),
    allow(dead_code, reason = "only the browser host inbox builds snapshots")
)]
pub(crate) struct HostSnapshot {
    /// Validated host palette.
    pub(crate) palette: Option<Palette>,
    /// Interface locale; unknown tags already mapped to `Locale::EnUs`.
    pub(crate) locale: Option<Locale>,
    /// Reduced-motion flag.
    pub(crate) reduced_motion: Option<bool>,
    /// View-state string that passed the whole-string checks, at most 2,000 bytes.
    pub(crate) view_state: Option<String>,
    /// Whether `set_palette` has been called at least once.
    pub(crate) connected: bool,
    /// Text typed into the host page's search field, at most `MAX_SEARCH_BYTES` bytes.
    pub(crate) search_text: Option<String>,
}
