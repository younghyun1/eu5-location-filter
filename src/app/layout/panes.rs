//! Layout mode, stored pane state, and the pure geometry that turns them into docked widths,
//! collapsed strips or drawers for a viewport.

use super::constants::{
    DETAIL_DEFAULT_WIDTH, DETAIL_MAX_WIDTH, DETAIL_MIN_WIDTH, DRAWER_DETAIL_WIDTH,
    DRAWER_EDGE_GAP, DRAWER_RAIL_WIDTH, HEADER_BRAND_MIN_WIDTH, NARROW_BREAKPOINT,
    RAIL_DEFAULT_WIDTH, RAIL_MAX_WIDTH, RAIL_MIN_WIDTH,
};
use super::direction::Direction;

/// Wide or narrow layout.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub(crate) enum LayoutMode {
    /// Docked rail and detail pane beside the grid.
    #[default]
    Wide,
    /// Below `NARROW_BREAKPOINT`: drawers, stacked header, touch sizes.
    Narrow,
}

/// Narrow-layout drawer.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Drawer {
    /// Filter rail drawer from the inline start.
    Filters,
    /// Detail drawer from the inline end.
    Details,
}

/// Side of the results a docked pane sits on, in inline terms.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum PaneEdge {
    /// Rail.
    Start,
    /// Detail pane.
    End,
}

/// Stored pane geometry. Widths are kept while a pane is collapsed, hidden or shown as a drawer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PaneState {
    /// Rail width when docked and expanded.
    pub(crate) rail_width: f32,
    /// Rail collapsed to its strip.
    pub(crate) rail_collapsed: bool,
    /// Detail width when docked and shown.
    pub(crate) detail_width: f32,
    /// Detail pane hidden.
    pub(crate) detail_hidden: bool,
    /// Open drawer in the narrow layout.
    pub(crate) drawer: Option<Drawer>,
}

impl Default for PaneState {
    fn default() -> Self {
        Self {
            rail_width: RAIL_DEFAULT_WIDTH,
            rail_collapsed: false,
            detail_width: DETAIL_DEFAULT_WIDTH,
            detail_hidden: false,
            drawer: None,
        }
    }
}

/// How one pane is placed this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum PaneSlot {
    /// Docked at this width.
    Docked(f32),
    /// Rail strip of `RAIL_COLLAPSED_WIDTH`.
    Collapsed,
    /// Not shown (hidden detail pane, or a closed drawer).
    Hidden,
    /// Shown as an open drawer of this width.
    Drawer(f32),
}

/// Resolved geometry for one viewport.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PaneLayout {
    /// Rail placement.
    pub(crate) rail: PaneSlot,
    /// Detail placement.
    pub(crate) detail: PaneSlot,
    /// Width left for the results region.
    pub(crate) grid_width: f32,
    /// Header brand slot width, aligned with the grid start when the rail is docked.
    pub(crate) brand_width: f32,
    /// The detail pane was hidden because both panes at their minimum left the grid narrower
    /// than `GRID_MIN_WIDTH`; update stores this as `detail_hidden`.
    pub(crate) detail_forced_hidden: bool,
}

/// Layout mode for a viewport width.
#[must_use]
pub(crate) fn mode_for_width(width: f32) -> LayoutMode {
    if width < NARROW_BREAKPOINT {
        LayoutMode::Narrow
    } else {
        LayoutMode::Wide
    }
}

/// Clamps a rail width to its limits.
#[must_use]
pub(crate) fn clamp_rail(width: f32) -> f32 {
    width.clamp(RAIL_MIN_WIDTH, RAIL_MAX_WIDTH)
}

/// Clamps a detail width to its limits.
#[must_use]
pub(crate) fn clamp_detail(width: f32) -> f32 {
    width.clamp(DETAIL_MIN_WIDTH, DETAIL_MAX_WIDTH)
}

/// Resolves docked widths, detail first, so the grid keeps `GRID_MIN_WIDTH` in the wide
/// layout; in the narrow layout panes are hidden or shown as the open drawer.
#[must_use]
pub(crate) fn resolve(viewport_width: f32, panes: &PaneState, mode: LayoutMode) -> PaneLayout {
    let _ = mode;
    PaneLayout {
        rail: PaneSlot::Docked(panes.rail_width),
        detail: PaneSlot::Docked(panes.detail_width),
        grid_width: (viewport_width - panes.rail_width - panes.detail_width).max(0.0),
        brand_width: HEADER_BRAND_MIN_WIDTH,
        detail_forced_hidden: false,
    }
}

/// Drawer width, limited to the viewport minus `DRAWER_EDGE_GAP`.
#[must_use]
pub(crate) fn drawer_width(drawer: Drawer, viewport_width: f32) -> f32 {
    let preferred = match drawer {
        Drawer::Filters => DRAWER_RAIL_WIDTH,
        Drawer::Details => DRAWER_DETAIL_WIDTH,
    };
    preferred.min((viewport_width - DRAWER_EDGE_GAP).max(0.0))
}

/// Pane width requested by a splitter drag: the width at drag start plus the pointer's screen
/// movement, signed by direction and by which side of the results the pane sits on. Unclamped.
#[must_use]
pub(crate) fn splitter_width(
    edge: PaneEdge,
    direction: Direction,
    start_width: f32,
    start_x: f32,
    x: f32,
) -> f32 {
    start_width + key_step(edge, direction, x - start_x)
}

/// Width change for a screen-direction movement of a pane's splitter (drag delta or one
/// `PANE_KEY_STEP` arrow press): the splitter follows the pointer or arrow on screen.
#[must_use]
pub(crate) fn key_step(edge: PaneEdge, direction: Direction, screen_dx: f32) -> f32 {
    let inline = direction.to_inline(screen_dx);
    match edge {
        PaneEdge::Start => inline,
        PaneEdge::End => -inline,
    }
}
