//! Transient surfaces, notices and keyboard focus bookkeeping.

use iced::Animation;
use iced::widget::Id;

/// Menus and dialogs; at most one is open at a time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Surface {
    /// Column visibility popover under the results toolbar.
    ColumnMenu,
    /// Desktop Language menu.
    LanguageMenu,
    /// Desktop Appearance menu.
    AppearanceMenu,
    /// About dialog over a scrim.
    About,
}

/// Non-modal message under the header; the interface keeps working underneath.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Notice {
    /// A language pack failed to load; English names are shown.
    LanguagePack,
    /// A language pack belongs to another data build and was not used.
    LanguagePackMismatch,
    /// A font pack failed to load; some text may not display.
    FontPack,
}

/// Open surfaces and their animation.
#[derive(Clone, Debug)]
pub(crate) struct OverlayState {
    /// Open menu or dialog.
    pub(crate) surface: Option<Surface>,
    /// Whether the About dialog shows the full licence texts.
    pub(crate) licenses_expanded: bool,
    /// Notice bar content.
    pub(crate) notice: Option<Notice>,
    /// Drawer slide and scrim fade; `true` is open. Durations are zero with reduced motion.
    pub(crate) drawer_motion: Animation<bool>,
}

impl Default for OverlayState {
    fn default() -> Self {
        Self {
            surface: None,
            licenses_expanded: false,
            notice: None,
            drawer_motion: Animation::new(false),
        }
    }
}

/// Keyboard regions F6 cycles through, in order.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum FocusRegion {
    /// Search and header buttons.
    #[default]
    Header,
    /// Filter rail.
    Rail,
    /// Results toolbar and grid.
    Grid,
    /// Detail pane.
    Detail,
}

/// Focus bookkeeping the runtime does not keep.
#[derive(Clone, Debug, Default)]
pub(crate) struct FocusState {
    /// Region of the last focus move made by the application.
    pub(crate) region: FocusRegion,
    /// Opener of the open surface or drawer, focused again when it closes.
    pub(crate) return_to: Option<Id>,
}
