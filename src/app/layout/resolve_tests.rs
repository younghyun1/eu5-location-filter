//! Pane resolution tests: wide layouts, drawers and brand width.

use super::panes::HEADER_MIN_WIDTH;
use super::{
    DESKTOP_MIN_WIDTH, DETAIL_DEFAULT_WIDTH, DETAIL_MAX_WIDTH, DETAIL_MIN_WIDTH, DRAWER_EDGE_GAP,
    DRAWER_RAIL_WIDTH, Drawer, GRID_MIN_WIDTH, HEADER_BRAND_MIN_WIDTH, LayoutMode,
    NARROW_BREAKPOINT, PaneLayout, PaneSlot, PaneState, RAIL_COLLAPSED_WIDTH, RAIL_DEFAULT_WIDTH,
    RAIL_MIN_WIDTH, SPLITTER_HIT_WIDTH, resolve,
};

/// Layout width a resolved slot takes in the wide body row, splitter included.
fn slot_extent(slot: PaneSlot) -> f32 {
    match slot {
        PaneSlot::Docked(width) => width + SPLITTER_HIT_WIDTH,
        PaneSlot::Collapsed => RAIL_COLLAPSED_WIDTH,
        PaneSlot::Hidden | PaneSlot::Drawer(_) => 0.0,
    }
}

/// Every resolved wide layout fills the viewport exactly when the grid is above zero.
fn assert_fills(viewport: f32, layout: &PaneLayout) {
    let used = slot_extent(layout.rail) + slot_extent(layout.detail) + layout.grid_width;
    assert!((used - viewport).abs() < 1e-3, "{layout:?} at {viewport}");
}

#[test]
fn wide_layout_keeps_stored_widths_when_room_allows() {
    let layout = resolve(1440.0, &PaneState::default(), LayoutMode::Wide);
    assert_eq!(layout.rail, PaneSlot::Docked(RAIL_DEFAULT_WIDTH));
    assert_eq!(layout.detail, PaneSlot::Docked(DETAIL_DEFAULT_WIDTH));
    assert!(!layout.detail_forced_hidden);
    assert_eq!(layout.grid_width, 1440.0 - 288.0 - 348.0);
    // 280 + 8 - 16 - 12: the search field starts where the grid starts.
    assert_eq!(layout.brand_width, 260.0);
    assert_fills(1440.0, &layout);
}

#[test]
fn wide_layout_shrinks_detail_before_rail() {
    let panes = PaneState::default();
    // 1050 - 420 = 630 for both panes: rail 288 stays, detail shrinks to 630 - 288 - 8.
    let layout = resolve(DESKTOP_MIN_WIDTH, &panes, LayoutMode::Wide);
    assert_eq!(layout.rail, PaneSlot::Docked(RAIL_DEFAULT_WIDTH));
    assert_eq!(layout.detail, PaneSlot::Docked(334.0));
    assert!(!layout.detail_forced_hidden);
    assert_eq!(layout.grid_width, GRID_MIN_WIDTH);
    assert_fills(DESKTOP_MIN_WIDTH, &layout);
    // 990 - 420 = 570: detail reaches its minimum, then the rail gives up 6 px.
    let layout = resolve(990.0, &panes, LayoutMode::Wide);
    assert_eq!(layout.detail, PaneSlot::Docked(DETAIL_MIN_WIDTH));
    assert_eq!(layout.rail, PaneSlot::Docked(274.0));
    assert_eq!(layout.grid_width, GRID_MIN_WIDTH);
    assert_fills(990.0, &layout);
}

#[test]
fn wide_layout_hides_detail_when_both_minimums_do_not_fit() {
    let panes = PaneState::default();
    // Exactly enough for both minimums: 240 + 8 + 280 + 8 + 420.
    let boundary = RAIL_MIN_WIDTH + DETAIL_MIN_WIDTH + 2.0 * SPLITTER_HIT_WIDTH + GRID_MIN_WIDTH;
    let layout = resolve(boundary, &panes, LayoutMode::Wide);
    assert!(!layout.detail_forced_hidden);
    assert_eq!(layout.rail, PaneSlot::Docked(RAIL_MIN_WIDTH));
    assert_eq!(layout.detail, PaneSlot::Docked(DETAIL_MIN_WIDTH));
    assert_fills(boundary, &layout);
    let layout = resolve(boundary - 1.0, &panes, LayoutMode::Wide);
    assert!(layout.detail_forced_hidden);
    assert_eq!(layout.detail, PaneSlot::Hidden);
    // Without the detail pane the rail regains its stored width.
    assert_eq!(layout.rail, PaneSlot::Docked(RAIL_DEFAULT_WIDTH));
    assert_fills(boundary - 1.0, &layout);
    let layout = resolve(NARROW_BREAKPOINT, &panes, LayoutMode::Wide);
    assert!(layout.detail_forced_hidden);
    assert_eq!(layout.grid_width, NARROW_BREAKPOINT - 288.0);
}

#[test]
fn collapsed_rail_leaves_room_for_detail_at_the_breakpoint() {
    let panes = PaneState {
        rail_collapsed: true,
        ..PaneState::default()
    };
    let layout = resolve(NARROW_BREAKPOINT, &panes, LayoutMode::Wide);
    assert_eq!(layout.rail, PaneSlot::Collapsed);
    assert!(!layout.detail_forced_hidden);
    // 768 - 420 - 40 - 8 = 300.
    assert_eq!(layout.detail, PaneSlot::Docked(300.0));
    assert_eq!(layout.grid_width, GRID_MIN_WIDTH);
    assert_eq!(layout.brand_width, HEADER_BRAND_MIN_WIDTH);
    assert_fills(NARROW_BREAKPOINT, &layout);
}

#[test]
fn hidden_detail_is_not_reported_as_forced() {
    let panes = PaneState {
        detail_hidden: true,
        ..PaneState::default()
    };
    let layout = resolve(NARROW_BREAKPOINT, &panes, LayoutMode::Wide);
    assert_eq!(layout.detail, PaneSlot::Hidden);
    assert!(!layout.detail_forced_hidden);
    assert_eq!(layout.rail, PaneSlot::Docked(RAIL_DEFAULT_WIDTH));
    assert_fills(NARROW_BREAKPOINT, &layout);
}

#[test]
fn wide_layout_ignores_drawers_and_bad_stored_widths() {
    let panes = PaneState {
        rail_width: f32::NAN,
        detail_width: 1e6,
        drawer: Some(Drawer::Filters),
        ..PaneState::default()
    };
    let layout = resolve(2000.0, &panes, LayoutMode::Wide);
    assert_eq!(layout.rail, PaneSlot::Docked(RAIL_DEFAULT_WIDTH));
    assert_eq!(layout.detail, PaneSlot::Docked(DETAIL_MAX_WIDTH));
    assert_fills(2000.0, &layout);
    let layout = resolve(f32::NAN, &PaneState::default(), LayoutMode::Wide);
    assert_eq!(layout.grid_width, 0.0);
}

#[test]
fn brand_yields_its_alignment_before_the_search_minimum() {
    let panes = PaneState {
        detail_hidden: true,
        ..PaneState::default()
    };
    let aligned = 260.0;
    let needed = HEADER_MIN_WIDTH + aligned - HEADER_BRAND_MIN_WIDTH;
    assert_eq!(
        resolve(needed, &panes, LayoutMode::Wide).brand_width,
        aligned
    );
    let below = resolve(needed - 1.0, &panes, LayoutMode::Wide).brand_width;
    assert_eq!(below, HEADER_BRAND_MIN_WIDTH);
    // A narrow docked rail never makes the brand smaller than its minimum.
    let thin = PaneState {
        rail_width: RAIL_MIN_WIDTH,
        ..panes
    };
    let layout = resolve(1440.0, &thin, LayoutMode::Wide);
    assert_eq!(
        layout.brand_width,
        HEADER_BRAND_MIN_WIDTH.max(RAIL_MIN_WIDTH - 20.0)
    );
}

#[test]
fn narrow_layout_shows_only_the_open_drawer() {
    let closed = resolve(400.0, &PaneState::default(), LayoutMode::Narrow);
    assert_eq!(closed.rail, PaneSlot::Hidden);
    assert_eq!(closed.detail, PaneSlot::Hidden);
    assert_eq!(closed.grid_width, 400.0);
    assert!(!closed.detail_forced_hidden);
    for (drawer, width) in [
        (Drawer::Filters, DRAWER_RAIL_WIDTH),
        (Drawer::Details, 352.0),
    ] {
        let panes = PaneState {
            drawer: Some(drawer),
            rail_collapsed: true,
            detail_hidden: true,
            ..PaneState::default()
        };
        let layout = resolve(400.0, &panes, LayoutMode::Narrow);
        let (open, other) = match drawer {
            Drawer::Filters => (layout.rail, layout.detail),
            Drawer::Details => (layout.detail, layout.rail),
        };
        assert_eq!(open, PaneSlot::Drawer(width));
        assert_eq!(other, PaneSlot::Hidden);
        assert_eq!(layout.grid_width, 400.0);
    }
    let tiny = PaneState {
        drawer: Some(Drawer::Filters),
        ..PaneState::default()
    };
    let layout = resolve(320.0, &tiny, LayoutMode::Narrow);
    assert_eq!(layout.rail, PaneSlot::Drawer(320.0 - DRAWER_EDGE_GAP));
}
