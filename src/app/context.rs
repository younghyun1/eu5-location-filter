//! Copyable view inputs every pane needs: palette, locale, direction, layout mode and platform.

use super::i18n::{self, Locale, Text};
use super::layout::{Direction, LayoutMode};
use super::theme::Palette;

/// Which build is running; decides whether desktop-only controls exist.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Platform {
    /// Native window with in-app language, appearance, retry and rebuild controls.
    Desktop,
    /// Sandboxed browser frame driven by the host page.
    Browser,
}

impl Platform {
    /// Whether the Language and Appearance menus and the load-error Retry and Rebuild controls
    /// are shown; the browser takes those settings from the host page instead.
    #[must_use]
    pub(crate) const fn has_desktop_controls(self) -> bool {
        matches!(self, Self::Desktop)
    }
}

/// View inputs built once per `view` call from the application state and passed by value, so
/// panes do not depend on the whole state type for their styling and text.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ViewContext {
    /// Palette shown this frame, already interpolated while a fade runs.
    pub(crate) palette: Palette,
    /// Interface locale.
    pub(crate) locale: Locale,
    /// Layout direction of the locale.
    pub(crate) direction: Direction,
    /// Wide or narrow layout for the current viewport.
    pub(crate) mode: LayoutMode,
    /// Running build.
    pub(crate) platform: Platform,
}

impl ViewContext {
    /// Catalog text for the current locale.
    #[must_use]
    pub(crate) fn t(&self, text: Text) -> &'static str {
        i18n::t(self.locale, text)
    }

    /// Whether the narrow layout applies, where controls use touch sizes.
    #[must_use]
    pub(crate) const fn is_narrow(&self) -> bool {
        matches!(self.mode, LayoutMode::Narrow)
    }
}
