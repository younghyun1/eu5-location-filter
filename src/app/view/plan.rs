//! Pure decisions behind the root view: which transient surfaces are stacked over the body,
//! where popover menus are anchored, and the header height the notice bar sits under. They take
//! plain values instead of widgets so they can be unit tested.

use iced::Size;

use crate::app::layout::{
    BUTTON_HEIGHT, BUTTON_WIDTH, DIALOG_MARGIN, Drawer, GRID_TOOLBAR_HEIGHT, HEADER_BUTTON_WIDTH,
    HEADER_GAP, HEADER_HEIGHT, HEADER_PADDING_INLINE, LayoutMode, MENU_MAX_HEIGHT, MENU_OFFSET,
    MENU_WIDTH, NARROW_HEADER_HEIGHT, NARROW_HEADER_ROW_HEIGHT, NARROW_TOOLBAR_HEIGHT, PaneLayout,
    PaneSlot, SPLITTER_HIT_WIDTH,
};
use crate::app::state::{Notice, Surface};

/// Inline padding of the results toolbar in the wide layout. The toolbar pane places the
/// Columns button this far from its inline end, so the column menu uses the same value to line
/// up with the button's inline end edge.
pub(super) const TOOLBAR_PADDING_INLINE: f32 = 12.0;

/// Gap between the Columns and Show details buttons in the wide results toolbar.
pub(super) const TOOLBAR_GAP: f32 = 8.0;

/// A non-modal popover menu; the About dialog is modal and planned separately.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Menu {
    /// Column visibility menu under the results toolbar.
    Columns,
    /// Desktop Language menu under its header button.
    Language,
    /// Desktop Appearance menu under its header button.
    Appearance,
}

/// Surfaces stacked over the body this frame. Rendering follows the contract's draw order from
/// back to front: menu, notice bar, drawer with its scrim, About dialog; the loading and error
/// veils come last and are decided from the load phase directly.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SurfacePlan {
    /// Open popover menu.
    pub(super) menu: Option<Menu>,
    /// Notice bar under the header.
    pub(super) notice: Option<Notice>,
    /// Open narrow-layout drawer and its width.
    pub(super) drawer: Option<(Drawer, f32)>,
    /// Whether the About dialog is open.
    pub(super) about: bool,
}

/// Decides the surfaces for the open surface, the notice and the resolved pane layout. Drawers
/// exist only in the narrow layout, where `layout::resolve` reports the open one as
/// `PaneSlot::Drawer`.
#[must_use]
pub(super) fn plan(
    surface: Option<Surface>,
    notice: Option<Notice>,
    panes: &PaneLayout,
    mode: LayoutMode,
) -> SurfacePlan {
    let (menu, about) = match surface {
        Some(Surface::ColumnMenu) => (Some(Menu::Columns), false),
        Some(Surface::LanguageMenu) => (Some(Menu::Language), false),
        Some(Surface::AppearanceMenu) => (Some(Menu::Appearance), false),
        Some(Surface::About) => (None, true),
        None => (None, false),
    };
    let drawer = match mode {
        LayoutMode::Wide => None,
        LayoutMode::Narrow => match (panes.rail, panes.detail) {
            (PaneSlot::Drawer(width), _) => Some((Drawer::Filters, width)),
            (_, PaneSlot::Drawer(width)) => Some((Drawer::Details, width)),
            _ => None,
        },
    };
    SurfacePlan {
        menu,
        notice,
        drawer,
        about,
    }
}

/// Header height for the layout mode; the notice bar is placed directly below it.
#[must_use]
pub(super) const fn header_height(mode: LayoutMode) -> f32 {
    match mode {
        LayoutMode::Wide => HEADER_HEIGHT,
        LayoutMode::Narrow => NARROW_HEADER_HEIGHT,
    }
}

/// Where a popover menu is drawn: distance from the window top, distance of its inline end
/// edge from the window's inline end edge, and its size limits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct MenuAnchor {
    /// Distance from the window top.
    pub(super) top: f32,
    /// Distance from the window's inline end edge to the menu's inline end edge.
    pub(super) end_inset: f32,
    /// Menu width, limited to the viewport minus `DIALOG_MARGIN` on each side.
    pub(super) width: f32,
    /// Maximum menu height, limited to the space below `top` minus `DIALOG_MARGIN`.
    pub(super) max_height: f32,
}

/// Anchors a menu `MENU_OFFSET` below its opener and aligned to the opener's inline end. The
/// view cannot query an opener's bounds, so the position follows from the header and toolbar
/// metrics: header buttons sit at the header's inline end in the order Language, Appearance,
/// About; the Columns button sits at the toolbar's inline end, before Show details while the
/// detail pane is not docked, and the toolbar ends where the docked detail pane's splitter
/// begins. In the narrow layout menus keep `DIALOG_MARGIN` from the inline end.
#[must_use]
pub(super) fn menu_anchor(
    menu: Menu,
    mode: LayoutMode,
    panes: &PaneLayout,
    viewport: Size,
) -> MenuAnchor {
    let header_button_bottom = HEADER_HEIGHT - (HEADER_HEIGHT - BUTTON_HEIGHT) / 2.0;
    let (top, end_inset) = match (mode, menu) {
        (LayoutMode::Wide, Menu::Language) => (
            header_button_bottom + MENU_OFFSET,
            HEADER_PADDING_INLINE + 2.0 * (HEADER_BUTTON_WIDTH + HEADER_GAP),
        ),
        (LayoutMode::Wide, Menu::Appearance) => (
            header_button_bottom + MENU_OFFSET,
            HEADER_PADDING_INLINE + HEADER_BUTTON_WIDTH + HEADER_GAP,
        ),
        (LayoutMode::Wide, Menu::Columns) => {
            let (detail_inset, show_details) = match panes.detail {
                PaneSlot::Docked(width) => (width + SPLITTER_HIT_WIDTH, 0.0),
                PaneSlot::Collapsed | PaneSlot::Hidden | PaneSlot::Drawer(_) => {
                    (0.0, BUTTON_WIDTH + TOOLBAR_GAP)
                }
            };
            (
                HEADER_HEIGHT + GRID_TOOLBAR_HEIGHT + MENU_OFFSET,
                detail_inset + TOOLBAR_PADDING_INLINE + show_details,
            )
        }
        (LayoutMode::Narrow, Menu::Columns) => (
            NARROW_HEADER_HEIGHT + NARROW_TOOLBAR_HEIGHT + MENU_OFFSET,
            DIALOG_MARGIN,
        ),
        (LayoutMode::Narrow, Menu::Language | Menu::Appearance) => {
            (NARROW_HEADER_ROW_HEIGHT + MENU_OFFSET, DIALOG_MARGIN)
        }
    };
    MenuAnchor {
        top,
        end_inset,
        width: MENU_WIDTH.min((viewport.width - 2.0 * DIALOG_MARGIN).max(0.0)),
        max_height: MENU_MAX_HEIGHT.min((viewport.height - top - DIALOG_MARGIN).max(0.0)),
    }
}
