//! The top-level message and the interaction messages of panes, menus and keys.

use iced::Size;
use iced::time::Instant;

use super::host::HostMessage;
use super::load::LoadMessage;
use crate::app::i18n::Locale;
use crate::app::layout::Drawer;
use crate::app::links::Link;
use crate::app::state::Surface;
use crate::app::theme::{AccentPreset, Mode};
use crate::app::view_model::{Bound, ColumnId, FacetId, OptionValue, RangeField};
use crate::model::LocationId;

/// Top-level message; each variant is handled by one helper in `update/`.
// Messages are moved once from the event to `update`; boxing the larger payloads would add an
// allocation per pointer or host event for no saving.
#[allow(clippy::large_enum_variant)]
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
