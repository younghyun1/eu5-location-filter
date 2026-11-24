//! Windowing math tests: visible ranges, clamping, scroll into view, thumb geometry, hit testing
//! and mirrored coordinates, including empty, huge and non-finite inputs.

use super::window::{
    Thumb, VisibleRange, clamp_offset, column_at, edge_at, lines_to_pixels, max_offset, mirror,
    offset_for_thumb, page_rows, row_at, scroll_into_view, span_into_view, thumb, visible_rows,
};
use crate::app::layout::{Direction, mirror_x};

const ROW: f32 = 32.0;
const WIDTHS: [f32; 3] = [100.0, 50.0, 200.0];

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 1e-3,
        "expected {expected}, got {actual}"
    );
}

fn range(start: usize, end: usize) -> VisibleRange {
    VisibleRange { start, end }
}

#[test]
fn visible_rows_cover_partial_rows_at_both_edges() {
    assert_eq!(visible_rows(0.0, 100.0, ROW, 1000), range(0, 4));
    assert_eq!(visible_rows(50.0, 100.0, ROW, 1000), range(1, 5));
    assert_eq!(visible_rows(64.0, 64.0, ROW, 1000), range(2, 4));
}

#[test]
fn visible_rows_clamp_to_count_and_offsets() {
    assert_eq!(visible_rows(0.0, 100.0, ROW, 3), range(0, 3));
    assert_eq!(visible_rows(10_000.0, 100.0, ROW, 10), range(10, 10));
    assert!(visible_rows(10_000.0, 100.0, ROW, 10).is_empty());
    assert_eq!(visible_rows(-40.0, 100.0, ROW, 1000), range(0, 4));
}

#[test]
fn visible_rows_are_empty_for_unusable_sizes() {
    for (viewport, row_height) in [
        (0.0, ROW),
        (-1.0, ROW),
        (f32::NAN, ROW),
        (f32::INFINITY, ROW),
        (100.0, 0.0),
        (100.0, -ROW),
        (100.0, f32::NAN),
        (100.0, f32::INFINITY),
    ] {
        assert_eq!(
            visible_rows(0.0, viewport, row_height, 1000),
            VisibleRange::default()
        );
    }
    assert_eq!(visible_rows(0.0, 100.0, ROW, 0), VisibleRange::default());
}

#[test]
fn visible_rows_treat_non_finite_offsets_as_zero() {
    assert_eq!(visible_rows(f32::NAN, 100.0, ROW, 1000), range(0, 4));
    assert_eq!(
        visible_rows(f32::NEG_INFINITY, 100.0, ROW, 1000),
        range(0, 4)
    );
    assert_eq!(visible_rows(f32::INFINITY, 100.0, ROW, 1000), range(0, 4));
}

#[test]
fn visible_rows_stay_bounded_for_huge_inputs() {
    assert_eq!(visible_rows(0.0, 100.0, ROW, usize::MAX), range(0, 4));
    let far = visible_rows(f32::MAX, f32::MAX, ROW, usize::MAX);
    assert!(far.start <= far.end);
    let window = visible_rows(1.0e9, 900.0, ROW, usize::MAX);
    assert!(window.len() <= 30, "{window:?}");
    let all = visible_rows(0.0, 900.0, ROW, 28_573);
    assert_eq!(all.len(), 29);
}

#[test]
fn visible_range_length_never_underflows() {
    assert_eq!(range(5, 2).len(), 0);
    assert!(range(5, 2).is_empty());
    assert_eq!(range(2, 5).len(), 3);
    assert!(!range(2, 5).is_empty());
}

#[test]
fn max_offset_is_zero_when_content_fits_or_is_not_finite() {
    assert_close(max_offset(1000.0, 300.0), 700.0);
    assert_close(max_offset(300.0, 1000.0), 0.0);
    assert_close(max_offset(f32::NAN, 300.0), 0.0);
    assert_close(max_offset(f32::INFINITY, 300.0), 0.0);
    assert_close(max_offset(1000.0, f32::NAN), 0.0);
    assert_close(max_offset(1000.0, -50.0), 1000.0);
}

#[test]
fn clamp_offset_keeps_offsets_in_range() {
    assert_close(clamp_offset(-5.0, 1000.0, 300.0), 0.0);
    assert_close(clamp_offset(250.0, 1000.0, 300.0), 250.0);
    assert_close(clamp_offset(800.0, 1000.0, 300.0), 700.0);
    assert_close(clamp_offset(800.0, 100.0, 300.0), 0.0);
    assert_close(clamp_offset(f32::NAN, 1000.0, 300.0), 0.0);
    assert_close(clamp_offset(f32::INFINITY, 1000.0, 300.0), 0.0);
    assert_close(clamp_offset(50.0, f32::NAN, 300.0), 0.0);
}

#[test]
fn row_at_maps_body_y_to_rows() {
    assert_eq!(row_at(0.0, 0.0, ROW, 10), Some(0));
    assert_eq!(row_at(31.9, 0.0, ROW, 10), Some(0));
    assert_eq!(row_at(32.0, 0.0, ROW, 10), Some(1));
    assert_eq!(row_at(0.0, 40.0, ROW, 10), Some(1));
    assert_eq!(row_at(319.9, 0.0, ROW, 10), Some(9));
    assert_eq!(row_at(320.0, 0.0, ROW, 10), None);
}

#[test]
fn row_at_rejects_unusable_input() {
    assert_eq!(row_at(-1.0, 0.0, ROW, 10), None);
    assert_eq!(row_at(f32::NAN, 0.0, ROW, 10), None);
    assert_eq!(row_at(f32::INFINITY, 0.0, ROW, 10), None);
    assert_eq!(row_at(10.0, 0.0, 0.0, 10), None);
    assert_eq!(row_at(10.0, 0.0, f32::NAN, 10), None);
    assert_eq!(row_at(10.0, 0.0, ROW, 0), None);
    assert_eq!(row_at(10.0, f32::NAN, ROW, 10), Some(0));
    assert_eq!(row_at(10.0, -100.0, ROW, 10), None);
    assert_eq!(row_at(1.0e9, 0.0, ROW, usize::MAX), Some(31_250_000));
    assert_eq!(row_at(f32::MAX, 0.0, ROW, 10), None);
}

#[test]
fn scroll_into_view_moves_the_least() {
    assert_close(scroll_into_view(64.0, 3, ROW, 128.0), 64.0);
    assert_close(scroll_into_view(64.0, 1, ROW, 128.0), 32.0);
    assert_close(scroll_into_view(64.0, 6, ROW, 128.0), 96.0);
    assert_close(scroll_into_view(0.0, 4, ROW, 128.0), 32.0);
    assert_close(scroll_into_view(0.0, 3, ROW, 128.0), 0.0);
}

#[test]
fn scroll_into_view_handles_tall_rows_and_bad_input() {
    assert_close(scroll_into_view(0.0, 2, ROW, 20.0), 64.0);
    assert_close(scroll_into_view(100.0, 2, ROW, 20.0), 64.0);
    assert_close(scroll_into_view(64.0, 3, 0.0, 128.0), 64.0);
    assert_close(scroll_into_view(64.0, 3, f32::NAN, 128.0), 64.0);
    assert_close(scroll_into_view(64.0, 3, ROW, f32::NAN), 64.0);
    assert_close(scroll_into_view(f32::NAN, 0, ROW, 128.0), 0.0);
    assert!(scroll_into_view(0.0, usize::MAX, ROW, 128.0).is_finite());
}

#[test]
fn span_into_view_moves_the_least() {
    assert_close(span_into_view(100.0, 150.0, 50.0, 200.0), 100.0);
    assert_close(span_into_view(100.0, 50.0, 50.0, 200.0), 50.0);
    assert_close(span_into_view(100.0, 280.0, 50.0, 200.0), 130.0);
    assert_close(span_into_view(100.0, 300.0, 400.0, 200.0), 300.0);
    assert_close(span_into_view(100.0, f32::NAN, 50.0, 200.0), 100.0);
    assert_close(span_into_view(100.0, 50.0, f32::INFINITY, 200.0), 100.0);
    assert_close(span_into_view(100.0, 50.0, 50.0, 0.0), 100.0);
    assert_close(span_into_view(f32::NAN, 50.0, 50.0, 200.0), 0.0);
}

#[test]
fn page_rows_is_visible_rows_minus_one_and_at_least_one() {
    assert_eq!(page_rows(320.0, ROW), 9);
    assert_eq!(page_rows(330.0, ROW), 9);
    assert_eq!(page_rows(64.0, ROW), 1);
    assert_eq!(page_rows(32.0, ROW), 1);
    assert_eq!(page_rows(10.0, ROW), 1);
    assert_eq!(page_rows(f32::NAN, ROW), 1);
    assert_eq!(page_rows(320.0, 0.0), 1);
    assert_eq!(page_rows(f32::INFINITY, ROW), 1);
}

#[test]
fn thumb_is_absent_when_content_fits_or_input_is_unusable() {
    assert_eq!(thumb(0.0, 200.0, 250.0, 250.0, 24.0), None);
    assert_eq!(thumb(0.0, 250.0, 250.0, 250.0, 24.0), None);
    assert_eq!(thumb(0.0, f32::NAN, 250.0, 250.0, 24.0), None);
    assert_eq!(thumb(0.0, f32::INFINITY, 250.0, 250.0, 24.0), None);
    assert_eq!(thumb(0.0, 1000.0, 0.0, 250.0, 24.0), None);
    assert_eq!(thumb(0.0, 1000.0, 250.0, 0.0, 24.0), None);
    assert_eq!(thumb(0.0, 1000.0, 250.0, f32::NAN, 24.0), None);
}

#[test]
fn thumb_length_is_proportional_and_bounded() {
    let at_top = Thumb {
        start: 0.0,
        length: 62.5,
    };
    assert_eq!(thumb(0.0, 1000.0, 250.0, 250.0, 24.0), Some(at_top));
    let at_end = Thumb {
        start: 187.5,
        length: 62.5,
    };
    assert_eq!(thumb(750.0, 1000.0, 250.0, 250.0, 24.0), Some(at_end));
    let long = Thumb {
        start: 0.0,
        length: 24.0,
    };
    assert_eq!(thumb(0.0, 1.0e6, 100.0, 100.0, 24.0), Some(long));
    let short_track = Thumb {
        start: 0.0,
        length: 10.0,
    };
    assert_eq!(thumb(5.0, 1000.0, 250.0, 10.0, 24.0), Some(short_track));
    assert_eq!(thumb(0.0, 1000.0, 250.0, 250.0, f32::NAN), Some(at_top));
}

#[test]
fn thumb_start_follows_clamped_offsets() {
    for (offset, expected) in [
        (f32::NAN, 0.0),
        (-10.0, 0.0),
        (375.0, 93.75),
        (5000.0, 187.5),
    ] {
        let found = thumb(offset, 1000.0, 250.0, 250.0, 24.0);
        assert!(found.is_some(), "offset {offset}");
        assert_close(found.unwrap_or_default().start, expected);
    }
    let rows = 28_573.0 * ROW;
    let found = thumb(rows, rows, 700.0, 700.0, 24.0);
    assert!(found.is_some());
    let found = found.unwrap_or_default();
    assert!(
        found.start >= 0.0 && found.start + found.length <= 700.001,
        "{found:?}"
    );
}

#[test]
fn offset_for_thumb_inverts_thumb() {
    for offset in [0.0, 100.0, 375.0, 750.0] {
        let found = thumb(offset, 1000.0, 250.0, 250.0, 24.0);
        assert!(found.is_some(), "offset {offset}");
        let start = found.unwrap_or_default().start;
        assert_close(offset_for_thumb(start, 1000.0, 250.0, 250.0, 24.0), offset);
    }
    assert_close(offset_for_thumb(500.0, 1000.0, 250.0, 250.0, 24.0), 750.0);
    assert_close(offset_for_thumb(-20.0, 1000.0, 250.0, 250.0, 24.0), 0.0);
    assert_close(offset_for_thumb(f32::NAN, 1000.0, 250.0, 250.0, 24.0), 0.0);
    assert_close(offset_for_thumb(50.0, 200.0, 250.0, 250.0, 24.0), 0.0);
    assert_close(offset_for_thumb(5.0, 1000.0, 250.0, 10.0, 24.0), 0.0);
}

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
