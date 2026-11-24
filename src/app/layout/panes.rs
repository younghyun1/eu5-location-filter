//! Layout mode, stored pane state, and the pure geometry that turns them into docked widths,
//! collapsed strips or drawers for a viewport.

use super::constants::{
    DETAIL_DEFAULT_WIDTH, DETAIL_MAX_WIDTH, DETAIL_MIN_WIDTH, DRAWER_DETAIL_WIDTH, DRAWER_EDGE_GAP,
    DRAWER_RAIL_WIDTH, GRID_MIN_WIDTH, HEADER_BRAND_MIN_WIDTH, HEADER_BUTTON_WIDTH, HEADER_GAP,
    HEADER_PADDING_INLINE, HEADER_STATUS_WIDTH, NARROW_BREAKPOINT, RAIL_COLLAPSED_WIDTH,
    RAIL_DEFAULT_WIDTH, RAIL_MAX_WIDTH, RAIL_MIN_WIDTH, SEARCH_MIN_WIDTH, SPLITTER_HIT_WIDTH,
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

/// Header buttons at the inline end: Language, Appearance and About on desktop, only About in
/// the browser build, whose host page owns language and appearance.
const HEADER_BUTTON_COUNT: f32 = if cfg!(all(feature = "web", target_family = "wasm")) {
    1.0
} else {
    3.0
};

/// Header width at the brand and search minimums: inline padding, brand, search, status slot,
/// the header buttons and one gap between each pair of items.
pub(super) const HEADER_MIN_WIDTH: f32 = 2.0 * HEADER_PADDING_INLINE
    + HEADER_BRAND_MIN_WIDTH
    + SEARCH_MIN_WIDTH
    + HEADER_STATUS_WIDTH
    + HEADER_BUTTON_COUNT * HEADER_BUTTON_WIDTH
    + (HEADER_BUTTON_COUNT + 2.0) * HEADER_GAP;

/// Resolves docked widths, detail first, so the grid keeps `GRID_MIN_WIDTH` in the wide
/// layout; in the narrow layout panes are hidden or shown as the open drawer.
///
/// Wide layout: a docked pane takes its clamped stored width plus `SPLITTER_HIT_WIDTH`, a
/// collapsed rail takes `RAIL_COLLAPSED_WIDTH` without a splitter, and a hidden detail pane
/// takes nothing. When the grid lacks room the detail pane shrinks toward its minimum, then the
/// rail; when both minimums still leave the grid too narrow the detail pane is hidden and
/// `detail_forced_hidden` is set, and the rail is fitted again without it. Stored widths are
/// never changed here, so a wider window restores them.
///
/// Narrow layout: the grid takes the whole width and a pane appears only as the drawer named by
/// `PaneState::drawer`, which update keeps set while a closing drawer slides out.
#[must_use]
pub(crate) fn resolve(viewport_width: f32, panes: &PaneState, mode: LayoutMode) -> PaneLayout {
    let viewport = if viewport_width.is_finite() {
        viewport_width.max(0.0)
    } else {
        0.0
    };
    match mode {
        LayoutMode::Narrow => resolve_narrow(viewport, panes),
        LayoutMode::Wide => resolve_wide(viewport, panes),
    }
}

/// Narrow placement: the open drawer, if any; everything else hidden.
fn resolve_narrow(viewport: f32, panes: &PaneState) -> PaneLayout {
    let slot = |drawer: Drawer| match panes.drawer {
        Some(open) if open == drawer => PaneSlot::Drawer(drawer_width(drawer, viewport)),
        Some(_) | None => PaneSlot::Hidden,
    };
    PaneLayout {
        rail: slot(Drawer::Filters),
        detail: slot(Drawer::Details),
        grid_width: viewport,
        // The narrow header omits the brand; the minimum keeps the value meaningful.
        brand_width: HEADER_BRAND_MIN_WIDTH,
        detail_forced_hidden: false,
    }
}

/// Wide placement with the shrink order of the layout document.
fn resolve_wide(viewport: f32, panes: &PaneState) -> PaneLayout {
    // Room the panes and their splitters may take while the grid keeps its minimum.
    let room = viewport - GRID_MIN_WIDTH;
    let rail_stored = clamp_rail(finite_or(panes.rail_width, RAIL_DEFAULT_WIDTH));
    let detail_stored = clamp_detail(finite_or(panes.detail_width, DETAIL_DEFAULT_WIDTH));
    let collapsed_extent = if panes.rail_collapsed {
        RAIL_COLLAPSED_WIDTH
    } else {
        0.0
    };

    let mut rail = (!panes.rail_collapsed).then_some(rail_stored);
    let mut detail = (!panes.detail_hidden).then_some(detail_stored);
    let mut detail_forced_hidden = false;

    if let Some(detail_width) = detail {
        let rail_extent = extent(rail) + collapsed_extent;
        let shrunk = fit(detail_width, DETAIL_MIN_WIDTH, room - rail_extent);
        detail = Some(shrunk);
        if let Some(rail_width) = rail {
            rail = Some(fit(rail_width, RAIL_MIN_WIDTH, room - extent(detail)));
        }
        if extent(rail) + collapsed_extent + extent(detail) > room {
            detail = None;
            detail_forced_hidden = true;
            rail = rail.map(|_| fit(rail_stored, RAIL_MIN_WIDTH, room));
        }
    } else if let Some(rail_width) = rail {
        rail = Some(fit(rail_width, RAIL_MIN_WIDTH, room));
    }

    let rail_slot = match rail {
        Some(width) => PaneSlot::Docked(width),
        None => PaneSlot::Collapsed,
    };
    let detail_slot = match detail {
        Some(width) => PaneSlot::Docked(width),
        None => PaneSlot::Hidden,
    };
    let used = extent(rail) + collapsed_extent + extent(detail);
    PaneLayout {
        rail: rail_slot,
        detail: detail_slot,
        grid_width: (viewport - used).max(0.0),
        brand_width: brand_width(viewport, rail),
        detail_forced_hidden,
    }
}

/// Header brand slot: aligned with the grid start while the rail is docked, unless the header
/// would then lack room for the search field at its minimum, in which case the brand yields
/// first.
fn brand_width(viewport: f32, rail: Option<f32>) -> f32 {
    match rail {
        Some(width) => {
            let aligned = (width + SPLITTER_HIT_WIDTH - HEADER_PADDING_INLINE - HEADER_GAP)
                .max(HEADER_BRAND_MIN_WIDTH);
            if viewport >= HEADER_MIN_WIDTH + aligned - HEADER_BRAND_MIN_WIDTH {
                aligned
            } else {
                HEADER_BRAND_MIN_WIDTH
            }
        }
        None => HEADER_BRAND_MIN_WIDTH,
    }
}

/// Layout width of a docked pane including its splitter; nothing for an absent pane.
fn extent(width: Option<f32>) -> f32 {
    match width {
        Some(width) => width + SPLITTER_HIT_WIDTH,
        None => 0.0,
    }
}

/// Largest pane width not above `preferred` and not below `minimum` whose width plus splitter
/// fits in `room`.
fn fit(preferred: f32, minimum: f32, room: f32) -> f32 {
    preferred.min(room - SPLITTER_HIT_WIDTH).max(minimum)
}

/// `value` when finite, otherwise `fallback`; `f32::clamp` would pass NaN through.
fn finite_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() { value } else { fallback }
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
