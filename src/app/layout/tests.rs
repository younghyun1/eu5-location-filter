//! Direction mirroring, layout mode, clamp and splitter tests.

use super::panes::HEADER_MIN_WIDTH;
use super::{
    DESKTOP_MIN_WIDTH, DETAIL_MAX_WIDTH, DETAIL_MIN_WIDTH, DRAWER_DETAIL_WIDTH, DRAWER_EDGE_GAP,
    DRAWER_RAIL_WIDTH, Direction, Drawer, LayoutMode, NARROW_BREAKPOINT, PANE_KEY_STEP, PaneEdge,
    RAIL_MAX_WIDTH, RAIL_MIN_WIDTH, clamp_detail, clamp_rail, drawer_width, key_step, mirror_x,
    mode_for_width, splitter_width,
};
use crate::app::widget::window::mirror;
use iced::alignment::Horizontal;

const BOTH: [Direction; 2] = [Direction::Ltr, Direction::Rtl];

#[test]
fn direction_helpers_mirror_alignment_and_deltas() {
    assert!(!Direction::Ltr.is_rtl());
    assert!(Direction::Rtl.is_rtl());
    assert_eq!(Direction::default(), Direction::Ltr);
    assert_eq!(Direction::Ltr.start(), Horizontal::Left);
    assert_eq!(Direction::Ltr.end(), Horizontal::Right);
    assert_eq!(Direction::Rtl.start(), Horizontal::Right);
    assert_eq!(Direction::Rtl.end(), Horizontal::Left);
    assert_eq!(Direction::Ltr.to_inline(5.0), 5.0);
    assert_eq!(Direction::Rtl.to_inline(5.0), -5.0);
    assert_ne!(Direction::Ltr.text_start(), Direction::Rtl.text_start());
}

#[test]
fn mirror_x_matches_the_window_mirroring_in_both_directions() {
    for direction in BOTH {
        for (x, width) in [(0.0, 10.0), (25.0, 40.0), (90.0, 10.0)] {
            let expected = mirror(direction.is_rtl(), 100.0, 100.0, x, width);
            assert_eq!(mirror_x(direction, 100.0, 100.0, x, width), expected);
        }
    }
    assert_eq!(mirror_x(Direction::Ltr, 100.0, 100.0, 0.0, 10.0), 100.0);
    assert_eq!(mirror_x(Direction::Rtl, 100.0, 100.0, 0.0, 10.0), 190.0);
}

#[test]
fn layout_mode_switches_at_the_breakpoint() {
    assert_eq!(mode_for_width(NARROW_BREAKPOINT - 0.5), LayoutMode::Narrow);
    assert_eq!(mode_for_width(NARROW_BREAKPOINT), LayoutMode::Wide);
    assert_eq!(mode_for_width(0.0), LayoutMode::Narrow);
    assert_eq!(mode_for_width(DESKTOP_MIN_WIDTH), LayoutMode::Wide);
}

#[test]
fn clamps_hold_pane_limits() {
    assert_eq!(clamp_rail(0.0), RAIL_MIN_WIDTH);
    assert_eq!(clamp_rail(1e6), RAIL_MAX_WIDTH);
    assert_eq!(clamp_rail(300.0), 300.0);
    assert_eq!(clamp_detail(0.0), DETAIL_MIN_WIDTH);
    assert_eq!(clamp_detail(1e6), DETAIL_MAX_WIDTH);
    assert_eq!(clamp_detail(400.0), 400.0);
}

#[test]
fn drawer_widths_leave_the_edge_gap() {
    assert_eq!(drawer_width(Drawer::Filters, 1000.0), DRAWER_RAIL_WIDTH);
    assert_eq!(drawer_width(Drawer::Details, 1000.0), DRAWER_DETAIL_WIDTH);
    assert_eq!(
        drawer_width(Drawer::Filters, 320.0),
        320.0 - DRAWER_EDGE_GAP
    );
    assert_eq!(
        drawer_width(Drawer::Details, 320.0),
        320.0 - DRAWER_EDGE_GAP
    );
    assert_eq!(drawer_width(Drawer::Details, 10.0), 0.0);
}

#[test]
fn splitters_follow_the_pointer_on_screen_in_both_directions() {
    // Rail on the left (LTR): dragging right widens it; on the right (RTL) dragging right narrows.
    assert_eq!(
        splitter_width(PaneEdge::Start, Direction::Ltr, 280.0, 100.0, 130.0),
        310.0
    );
    assert_eq!(
        splitter_width(PaneEdge::Start, Direction::Rtl, 280.0, 100.0, 130.0),
        250.0
    );
    // Detail on the right (LTR): dragging right narrows it; on the left (RTL) it widens.
    assert_eq!(
        splitter_width(PaneEdge::End, Direction::Ltr, 340.0, 100.0, 130.0),
        310.0
    );
    assert_eq!(
        splitter_width(PaneEdge::End, Direction::Rtl, 340.0, 100.0, 130.0),
        370.0
    );
    for direction in BOTH {
        for edge in [PaneEdge::Start, PaneEdge::End] {
            assert_eq!(splitter_width(edge, direction, 300.0, 50.0, 50.0), 300.0);
            let right = key_step(edge, direction, PANE_KEY_STEP);
            let left = key_step(edge, direction, -PANE_KEY_STEP);
            assert_eq!(right, -left);
            assert_eq!(right.abs(), PANE_KEY_STEP);
        }
    }
    assert_eq!(
        key_step(PaneEdge::Start, Direction::Ltr, PANE_KEY_STEP),
        PANE_KEY_STEP
    );
    assert_eq!(
        key_step(PaneEdge::Start, Direction::Rtl, PANE_KEY_STEP),
        -PANE_KEY_STEP
    );
    assert_eq!(
        key_step(PaneEdge::End, Direction::Ltr, PANE_KEY_STEP),
        -PANE_KEY_STEP
    );
    assert_eq!(
        key_step(PaneEdge::End, Direction::Rtl, PANE_KEY_STEP),
        PANE_KEY_STEP
    );
}

#[test]
fn header_minimum_matches_the_layout_document() {
    let expected = if cfg!(all(feature = "web", target_family = "wasm")) {
        796.0
    } else {
        1028.0
    };
    assert_eq!(HEADER_MIN_WIDTH, expected);
}
