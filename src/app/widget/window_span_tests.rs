//! Windowing math tests for column hit testing, resize edges, mirrored coordinates and wheel
//! lines.

use super::window::{column_at, edge_at, lines_to_pixels, mirror};
use super::window_tests::assert_close;
use crate::app::layout::{Direction, mirror_x};

const WIDTHS: [f32; 3] = [100.0, 50.0, 200.0];

#[test]
fn column_at_finds_spans_in_display_order() {
    assert_eq!(column_at(0.0, &WIDTHS), Some(0));
    assert_eq!(column_at(99.9, &WIDTHS), Some(0));
    assert_eq!(column_at(100.0, &WIDTHS), Some(1));
    assert_eq!(column_at(349.9, &WIDTHS), Some(2));
    assert_eq!(column_at(350.0, &WIDTHS), None);
    assert_eq!(column_at(-1.0, &WIDTHS), None);
    assert_eq!(column_at(f32::NAN, &WIDTHS), None);
    assert_eq!(column_at(10.0, &[]), None);
    assert_eq!(column_at(100.0, &[100.0, 0.0, 50.0]), Some(2));
    assert_eq!(column_at(100.0, &[100.0, f32::NAN, 50.0]), Some(2));
}

#[test]
fn edge_at_finds_inline_end_edges() {
    assert_eq!(edge_at(100.0, &WIDTHS, 8.0), Some(0));
    assert_eq!(edge_at(96.0, &WIDTHS, 8.0), Some(0));
    assert_eq!(edge_at(104.0, &WIDTHS, 8.0), Some(0));
    assert_eq!(edge_at(95.9, &WIDTHS, 8.0), None);
    assert_eq!(edge_at(150.0, &WIDTHS, 8.0), Some(1));
    assert_eq!(edge_at(352.0, &WIDTHS, 8.0), Some(2));
    assert_eq!(edge_at(0.0, &WIDTHS, 8.0), None);
    assert_eq!(edge_at(100.0, &WIDTHS, 0.0), None);
    assert_eq!(edge_at(100.0, &WIDTHS, f32::NAN), None);
    assert_eq!(edge_at(f32::NAN, &WIDTHS, 8.0), None);
    assert_eq!(edge_at(100.0, &[], 8.0), None);
}

#[test]
fn edge_at_prefers_the_nearer_edge() {
    assert_eq!(edge_at(102.0, &[100.0, 4.0], 8.0), Some(0));
    assert_eq!(edge_at(103.0, &[100.0, 4.0], 8.0), Some(1));
    assert_eq!(edge_at(101.0, &[100.0, 4.0], 8.0), Some(0));
}

#[test]
fn mirror_reflects_spans_for_right_to_left() {
    assert_close(mirror(false, 10.0, 200.0, 30.0, 50.0), 40.0);
    assert_close(mirror(true, 10.0, 200.0, 30.0, 50.0), 130.0);
    assert_close(mirror(true, 10.0, 200.0, 0.0, 200.0), 10.0);
    let left = mirror(true, 10.0, 200.0, 30.0, 50.0);
    assert_close(mirror(true, 10.0, 200.0, left - 10.0, 50.0), 40.0);
}

#[test]
fn mirror_matches_layout_mirror_x() {
    for (x, width) in [(0.0, 10.0), (30.0, 50.0), (199.0, 1.0), (-20.0, 40.0)] {
        assert_close(
            mirror(false, 10.0, 200.0, x, width),
            mirror_x(Direction::Ltr, 10.0, 200.0, x, width),
        );
        assert_close(
            mirror(true, 10.0, 200.0, x, width),
            mirror_x(Direction::Rtl, 10.0, 200.0, x, width),
        );
    }
}

#[test]
fn wheel_lines_use_the_scrollable_factor() {
    assert_close(lines_to_pixels(1.0), 60.0);
    assert_close(lines_to_pixels(-2.5), -150.0);
    assert_close(lines_to_pixels(0.0), 0.0);
}
