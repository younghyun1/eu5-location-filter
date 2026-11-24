//! Grid geometry tests: regions in both directions, the visible column strip, screen and inline
//! mapping, hit testing of headers, resize edges, rows and lanes, and label placement.

use iced::{Color, Point, Rectangle};

use super::geometry::{Frame, Strip};
use super::metrics::{Hit, Metrics};
use super::paint::{self, Side, Style};
use crate::app::layout::{Direction, GRID_INLINE_INSET};
use crate::app::theme::MONO;
use crate::app::view_model::{CellAlign, ColumnId, Columns};

const BOUNDS: Rectangle = Rectangle {
    x: 0.0,
    y: 0.0,
    width: 800.0,
    height: 600.0,
};

fn metrics(direction: Direction, rows: usize) -> Metrics {
    Metrics {
        frame: Frame::new(BOUNDS, direction),
        strip: Strip::new(&Columns::default()),
        rows,
        row_height: 32.0,
        offset_x: 0.0,
        offset_y: 0.0,
    }
}

fn assert_rect(actual: Rectangle, expected: Rectangle) {
    let close = |a: f32, b: f32| (a - b).abs() < 1e-3;
    assert!(
        close(actual.x, expected.x)
            && close(actual.y, expected.y)
            && close(actual.width, expected.width)
            && close(actual.height, expected.height),
        "expected {expected:?}, got {actual:?}"
    );
}

#[test]
fn frame_reserves_both_lanes_at_the_inline_and_block_end() {
    let ltr = Frame::new(BOUNDS, Direction::Ltr);
    assert_rect(
        ltr.header,
        Rectangle {
            x: 0.0,
            y: 0.0,
            width: 790.0,
            height: 36.0,
        },
    );
    assert_rect(
        ltr.body,
        Rectangle {
            x: 0.0,
            y: 36.0,
            width: 790.0,
            height: 554.0,
        },
    );
    assert_rect(
        ltr.vertical_lane,
        Rectangle {
            x: 790.0,
            y: 36.0,
            width: 10.0,
            height: 554.0,
        },
    );
    assert_rect(
        ltr.horizontal_lane,
        Rectangle {
            x: 0.0,
            y: 590.0,
            width: 790.0,
            height: 10.0,
        },
    );
    let rtl = Frame::new(BOUNDS, Direction::Rtl);
    assert_rect(
        rtl.body,
        Rectangle {
            x: 10.0,
            y: 36.0,
            width: 790.0,
            height: 554.0,
        },
    );
    assert_rect(
        rtl.vertical_lane,
        Rectangle {
            x: 0.0,
            y: 36.0,
            width: 10.0,
            height: 554.0,
        },
    );
}

#[test]
fn frame_never_produces_negative_sizes() {
    let tiny = Frame::new(
        Rectangle {
            x: 5.0,
            y: 5.0,
            width: 4.0,
            height: 20.0,
        },
        Direction::Ltr,
    );
    for rect in [
        tiny.header,
        tiny.body,
        tiny.vertical_lane,
        tiny.horizontal_lane,
    ] {
        assert!(rect.width >= 0.0 && rect.height >= 0.0, "{rect:?}");
    }
    let negative = Frame::new(
        Rectangle {
            x: 0.0,
            y: 0.0,
            width: -10.0,
            height: -10.0,
        },
        Direction::Rtl,
    );
    assert!(negative.body.width >= 0.0 && negative.body.height >= 0.0);
}

#[test]
fn screen_and_inline_x_round_trip_in_both_directions() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        let frame = Frame::new(BOUNDS, direction);
        for (inline, offset) in [(0.0, 0.0), (118.0, 0.0), (500.0, 120.0)] {
            let screen = frame.screen_x(inline, 0.0, offset);
            assert!((frame.inline_x(screen, offset) - inline).abs() < 1e-3);
        }
    }
    let rtl = Frame::new(BOUNDS, Direction::Rtl);
    assert!((rtl.screen_x(0.0, 10.0, 0.0) - 790.0).abs() < 1e-3);
    assert!((rtl.to_inline(5.0) + 5.0).abs() < 1e-3);
}

#[test]
fn strip_holds_default_columns_with_the_ported_width() {
    let strip = Strip::new(&Columns::default());
    assert_eq!(strip.len(), 11);
    assert!((strip.content_width() - 1346.0).abs() < 1e-3);
    assert_eq!(strip.position(ColumnId::Name), Some(1));
    assert_eq!(strip.position(ColumnId::Continent), None);
    let name = strip.slot(1);
    assert_eq!(name.map(|slot| slot.id), Some(ColumnId::Name));
    assert_eq!(name.map(|slot| slot.start), Some(GRID_INLINE_INSET + 108.0));
    assert_eq!(strip.slot(11), None);
}

#[test]
fn strip_skips_hidden_columns_and_windows_by_offset() {
    let mut columns = Columns::default();
    assert!(columns.set_visible(ColumnId::Name, false));
    let strip = Strip::new(&columns);
    assert_eq!(strip.position(ColumnId::Name), None);
    assert_eq!(strip.position(ColumnId::Identifier), Some(1));
    let full = Strip::new(&Columns::default());
    let ids: Vec<ColumnId> = full.visible(0.0, 300.0).map(|slot| slot.id).collect();
    assert_eq!(ids, vec![ColumnId::Color, ColumnId::Name]);
    let scrolled: Vec<ColumnId> = full.visible(400.0, 100.0).map(|slot| slot.id).collect();
    assert_eq!(scrolled, vec![ColumnId::Identifier, ColumnId::Kind]);
}

#[test]
fn hit_testing_left_to_right() {
    let m = metrics(Direction::Ltr, 100);
    assert_eq!(m.hit(Point::new(118.0, 10.0)), Some(Hit::Edge(0)));
    assert_eq!(m.hit(Point::new(50.0, 10.0)), Some(Hit::Header(0)));
    assert_eq!(m.hit(Point::new(5.0, 10.0)), None);
    assert_eq!(m.hit(Point::new(100.0, 36.0 + 33.0)), Some(Hit::Row(1)));
    assert_eq!(m.hit(Point::new(795.0, 100.0)), Some(Hit::VerticalLane));
    assert_eq!(m.hit(Point::new(100.0, 595.0)), Some(Hit::HorizontalLane));
    assert_eq!(m.hit(Point::new(900.0, 100.0)), None);
}

#[test]
fn hit_testing_right_to_left_mirrors_columns() {
    let m = metrics(Direction::Rtl, 100);
    assert_eq!(m.hit(Point::new(682.0, 10.0)), Some(Hit::Edge(0)));
    assert_eq!(m.hit(Point::new(760.0, 10.0)), Some(Hit::Header(0)));
    assert_eq!(m.hit(Point::new(600.0, 10.0)), Some(Hit::Header(1)));
    assert_eq!(m.hit(Point::new(5.0, 100.0)), Some(Hit::VerticalLane));
    assert_eq!(m.hit(Point::new(400.0, 36.0)), Some(Hit::Row(0)));
}

#[test]
fn hit_testing_follows_offsets_and_row_count() {
    let mut m = metrics(Direction::Ltr, 3);
    assert_eq!(m.hit(Point::new(100.0, 36.0 + 100.0)), None);
    m.rows = 100;
    m.offset_y = 64.0;
    assert_eq!(m.hit(Point::new(100.0, 36.0)), Some(Hit::Row(2)));
    m.offset_x = 108.0;
    assert_eq!(m.hit(Point::new(10.0, 10.0)), Some(Hit::Edge(0)));
}

#[test]
fn metrics_clamp_offsets_and_place_thumbs() {
    let m = metrics(Direction::Ltr, 1000);
    assert!((m.content_height() - 32_000.0).abs() < 1e-3);
    assert!((m.clamp_y(1.0e9) - (32_000.0 - 554.0)).abs() < 1e-2);
    assert!((m.clamp_x(1.0e9) - (1346.0 - 790.0)).abs() < 1e-3);
    assert!(m.clamp_x(f32::NAN).abs() < 1e-6);
    assert!(m.vertical_thumb(0.0).is_some());
    assert!(m.horizontal_thumb(0.0).is_some());
    assert_eq!(metrics(Direction::Ltr, 5).vertical_thumb(0.0), None);
    let rows = m.visible_rows();
    assert_eq!((rows.start, rows.end), (0, 18));
    assert!((m.row_y(2) - (36.0 + 64.0)).abs() < 1e-3);
}

#[test]
fn sides_follow_logical_alignment_and_direction() {
    assert_eq!(Side::of(CellAlign::Start, false), Side::Left);
    assert_eq!(Side::of(CellAlign::End, false), Side::Right);
    assert_eq!(Side::of(CellAlign::Start, true), Side::Right);
    assert_eq!(Side::of(CellAlign::End, true), Side::Left);
}

#[test]
fn right_to_left_detection_only_flags_rtl_scripts() {
    assert!(!paint::may_be_rtl("Paris"));
    assert!(!paint::may_be_rtl("Zürich"));
    assert!(!paint::may_be_rtl("서울"));
    assert!(!paint::may_be_rtl(""));
    assert!(paint::may_be_rtl(
        "\u{0645}\u{0631}\u{062D}\u{0628}\u{0627}"
    ));
    assert!(paint::may_be_rtl("\u{05E9}\u{05DC}\u{05D5}\u{05DD}"));
    assert!(paint::may_be_rtl("12 \u{0644}\u{0627}"));
}

#[test]
fn left_to_right_labels_anchor_without_measuring() {
    let style = Style {
        font: MONO,
        size: 12.0,
        color: Color::BLACK,
    };
    let (x, align) = paint::anchor("1,234", Side::Right, 10.0, 110.0, style);
    assert!((x - 110.0).abs() < 1e-3);
    assert_eq!(align, iced::advanced::text::Alignment::Right);
    let (x, align) = paint::anchor("abc", Side::Center, 10.0, 110.0, style);
    assert!((x - 60.0).abs() < 1e-3);
    assert_eq!(align, iced::advanced::text::Alignment::Center);
    let (x, align) = paint::anchor("\u{0644}\u{0627}", Side::Left, 10.0, 110.0, style);
    assert!((x - 10.0).abs() < 1e-3);
    assert_eq!(align, iced::advanced::text::Alignment::Default);
}
