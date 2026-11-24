//! Every interaction the application handles, grouped by the update helper that owns it.
//!
//! iced requires messages to be `Clone + Send + 'static` on every target, so values that cannot
//! be cloned travel in a [`Handoff`] and errors travel as `Arc<AppError>`. Option and column
//! identities are typed enums, never display strings.

use std::sync::Arc;

use iced::Size;
use iced::time::Instant;

use super::handoff::Handoff;
use super::i18n::{FontPack, Locale, Text};
use super::layout::Drawer;
use super::links::Link;
use super::load::LoadedData;
use super::state::Surface;
use super::theme::{AccentPreset, Mode, Palette};
use super::view_model::{Bound, ColumnId, FacetId, OptionValue, RangeField};
use crate::AppError;
use crate::lang_pack::{GameLanguage, ValidatedLanguagePack};
use crate::model::LocationId;

/// Top-level message; each variant is handled by one helper in `update/`.
#[derive(Clone, Debug)]
pub(crate) enum Message {
    /// Search, facet selections, rail toggles and rail text fields.
    Filter(FilterMessage),
    /// Result grid selection, sorting and column resizing.
    Grid(GridMessage),
    /// Column visibility from the column menu.
    Column(ColumnMessage),
    /// Pane widths, rail collapse and detail hide.
    Pane(PaneMessage),
    /// Menus, dialogs, drawers, notices and About links.
    Overlay(OverlayMessage),
    /// Desktop language and appearance choices.
    Settings(SettingsMessage),
    /// Data, language pack and font pack loading.
    Load(LoadMessage),
    /// Browser host values drained from the inbox; never produced on desktop.
    Host(HostMessage),
    /// Application-level keyboard commands.
    Key(KeyCommand),
    /// Logical window size after a resize, which decides the layout mode.
    WindowResized(Size),
    /// One animation frame while a palette fade or drawer slide runs.
    Frame(Instant),
}

/// Filter rail and header search input.
#[derive(Clone, Debug)]
pub(crate) enum FilterMessage {
    /// Global search text, applied as typed.
    Search(String),
    /// One checklist option changed; `value` must belong to `facet`, otherwise it is ignored.
    Toggle {
        /// Facet group.
        facet: FacetId,
        /// Typed option value.
        value: OptionValue,
        /// New checked state.
        selected: bool,
    },
    /// Clear button of one facet group.
    ClearFacet(FacetId),
    /// Fold or unfold one facet group.
    Fold {
        /// Facet group.
        facet: FacetId,
        /// New folded state.
        folded: bool,
    },
    /// Option search text of one facet group.
    OptionQuery {
        /// Facet group.
        facet: FacetId,
        /// Raw query; folded once per change, not per frame.
        query: String,
    },
    /// Show impassable toggle.
    ShowImpassable(bool),
    /// Food-producing only toggle.
    FoodOnly(bool),
    /// Exact-colour field text; invalid text keeps the active filter and shows an inline error.
    Rgb(String),
    /// One bound field of a numeric range.
    Range {
        /// Range row.
        field: RangeField,
        /// Minimum or maximum field.
        bound: Bound,
        /// Raw field text.
        text: String,
    },
    /// Clear all: restores `FilterSet::land_only` and empties every field and query.
    ClearAll,
}

/// How a row selection was made; a pointer tap in the narrow layout also opens the detail
/// drawer, keyboard movement does not.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SelectionSource {
    /// Mouse click or touch tap.
    Pointer,
    /// Arrow, page, Home or End key.
    Keyboard,
}

/// Messages published by the result grid.
#[derive(Clone, Debug)]
pub(crate) enum GridMessage {
    /// A row was selected.
    Select {
        /// Selected location.
        id: LocationId,
        /// Pointer or keyboard.
        source: SelectionSource,
    },
    /// Header click or Enter on the active column; the same column again reverses.
    Sort(ColumnId),
    /// Header edge drag or Shift with an arrow; the width is unclamped and update clamps it.
    ResizeColumn {
        /// Resized column.
        column: ColumnId,
        /// Requested width in logical pixels.
        width: f32,
    },
    /// Space on a focused grid: show the detail pane, or open the detail drawer when narrow.
    ShowDetail(LocationId),
}

/// Column menu.
#[derive(Clone, Copy, Debug)]
pub(crate) enum ColumnMessage {
    /// Show or hide one column; hiding the last visible column is refused.
    SetVisible {
        /// Column.
        column: ColumnId,
        /// New visibility.
        visible: bool,
    },
}

/// Docked pane geometry.
#[derive(Clone, Copy, Debug)]
pub(crate) enum PaneMessage {
    /// Requested rail width from its splitter, unclamped.
    ResizeRail(f32),
    /// Requested detail width from its splitter, unclamped.
    ResizeDetail(f32),
    /// Collapse (`true`) or expand the rail.
    SetRailCollapsed(bool),
    /// Hide (`true`) or show the detail pane.
    SetDetailHidden(bool),
}

/// Transient surfaces.
#[derive(Clone, Copy, Debug)]
pub(crate) enum OverlayMessage {
    /// Open a menu or dialog, closing any other.
    Open(Surface),
    /// Toggle a surface from its opener: the Columns button also closes the column menu.
    Toggle(Surface),
    /// Close the topmost surface and return focus to its opener.
    Close,
    /// Open a narrow-layout drawer.
    OpenDrawer(Drawer),
    /// Close the open drawer.
    CloseDrawer,
    /// Expand or collapse the licence texts in the About dialog.
    SetLicensesExpanded(bool),
    /// Dismiss the notice bar.
    DismissNotice,
    /// Open an About link outside the application.
    OpenLink(Link),
}

/// Desktop Language and Appearance menus.
#[derive(Clone, Copy, Debug)]
pub(crate) enum SettingsMessage {
    /// Interface locale.
    Locale(Locale),
    /// Light or dark built-in palette.
    Mode(Mode),
    /// Accent preset.
    Accent(AccentPreset),
    /// Animations on (`true`) or off.
    Motion(bool),
}

/// Loading results and load-error controls.
#[derive(Clone, Debug)]
pub(crate) enum LoadMessage {
    /// The browser painted its first frame; decoding may start.
    Painted,
    /// Progress of the load with this generation; stale generations are ignored.
    Progress {
        /// Load generation that reported.
        generation: u64,
        /// Catalog text for the stage.
        stage: Text,
        /// Completed fraction in `0.0..=1.0`.
        fraction: f32,
    },
    /// Final result of the load with this generation.
    Finished {
        /// Load generation that finished.
        generation: u64,
        /// Prepared dataset and engine, or the failure.
        result: Result<Handoff<LoadedData>, Arc<AppError>>,
    },
    /// Desktop installation directory field text on the error card.
    InstallPath(String),
    /// Desktop Retry: load again with the entered directory.
    Retry,
    /// Desktop Rebuild data: import from the game files and replace the external bundles.
    Rebuild,
    /// Browser: compressed language pack bytes, validated inside update.
    LanguagePackFetched {
        /// Requested language.
        language: GameLanguage,
        /// Pack bytes or the fetch failure.
        result: Result<Arc<[u8]>, Arc<AppError>>,
    },
    /// Desktop: pack decoded and validated off the UI thread.
    LanguagePackValidated {
        /// Requested language.
        language: GameLanguage,
        /// Validated pack or the failure.
        result: Result<Handoff<ValidatedLanguagePack>, Arc<AppError>>,
    },
    /// A font pack was registered with the renderer, or failed to load.
    FontPackLoaded {
        /// Requested pack.
        pack: FontPack,
        /// Success or the failure.
        result: Result<(), Arc<AppError>>,
    },
}

/// Host values drained from the browser inbox in one wake.
#[derive(Clone, Debug)]
pub(crate) enum HostMessage {
    /// Latest value of every slot that changed since the previous drain.
    Update(HostSnapshot),
    /// Window-space bounds of the header search slot, published by the `Anchor` widget so the
    /// host page can place its real text field (which supports input methods) over it.
    SearchAnchor(iced::Rectangle),
}

/// One drain of the browser inbox; each slot holds only the latest value.
#[derive(Clone, Debug, Default)]
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

/// Application-level keyboard commands, read from events no widget captured (and Escape from
/// all events, because a focused text input captures it).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KeyCommand {
    /// Tab: next focusable control, wrapping inside an open surface.
    FocusNext,
    /// Shift+Tab: previous focusable control.
    FocusPrevious,
    /// Ctrl+F or Cmd+F: focus the search field and select its text.
    FocusSearch,
    /// F6: first control of the next region.
    NextRegion,
    /// Shift+F6: first control of the previous region.
    PreviousRegion,
    /// Escape: close the topmost surface and return focus to its opener.
    Escape,
}
