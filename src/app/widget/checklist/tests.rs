//! Checklist tests: list height, inline geometry in both directions, key navigation, the tap
//! test, script detection, and simulator flows that find rows by the labels `operate` reports.

use iced::{Point, Rectangle};

use super::geometry::{Geometry, Nav, has_rtl_script, is_tap, list_height, navigate};
use super::model::ChecklistModel;
use crate::app::layout::{
    CHECK_LABEL_GAP, CHECKBOX_SIZE, CHECKLIST_PADDING_INLINE, CHECKLIST_ROW_HEIGHT,
    SCROLLBAR_WIDTH, TOUCH_SLOP,
};

/// Row source over fixed labels and checked states; the option index is the row index.
struct Rows(Vec<(&'static str, bool)>);

impl ChecklistModel for Rows {
    fn row_count(&self) -> usize {
        self.0.len()
    }

    fn label(&self, row: usize) -> &str {
        self.0.get(row).map_or("", |(label, _)| *label)
    }

    fn is_checked(&self, row: usize) -> bool {
        self.0.get(row).is_some_and(|(_, checked)| *checked)
    }

    fn option(&self, row: usize) -> u32 {
        u32::try_from(row).unwrap_or(u32::MAX)
    }
}

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-3
}

const BOUNDS: Rectangle = Rectangle { x: 10.0, y: 20.0, width: 200.0, height: 196.0 };

#[test]
fn height_is_shown_rows_times_row_height() {
    assert!(close(list_height(10, 7, 28.0, false), 196.0));
    assert!(close(list_height(3, 7, 28.0, false), 84.0));
    assert!(close(list_height(26, 26, 28.0, false), 728.0));
}

#[test]
fn empty_list_keeps_one_row_only_for_empty_text() {
    assert!(close(list_height(0, 7, 28.0, true), 28.0));
    assert!(close(list_height(0, 7, 28.0, false), 0.0));
}

#[test]
fn invalid_row_height_falls_back_to_wide_metrics() {
    assert!(close(list_height(2, 7, 0.0, false), 2.0 * CHECKLIST_ROW_HEIGHT));
    assert!(close(list_height(2, 7, f32::NAN, false), 2.0 * CHECKLIST_ROW_HEIGHT));
}

#[test]
fn short_list_has_no_lane() {
    let geometry = Geometry::new(BOUNDS, 28.0, 3, false);
    assert!(!geometry.is_scrollable());
    assert!(geometry.lane().is_none());
    assert_eq!(geometry.rows_area(), BOUNDS);
    assert!(geometry.thumb_rect(0.0, false).is_none());
}

#[test]
fn lane_sits_at_the_inline_end() {
    let ltr = Geometry::new(BOUNDS, 28.0, 100, false);
    let lane = ltr.lane();
    assert!(lane.is_some_and(|lane| close(lane.x, BOUNDS.x + BOUNDS.width - SCROLLBAR_WIDTH)));
    assert!(close(ltr.rows_area().x, BOUNDS.x));
    assert!(close(ltr.rows_area().width, BOUNDS.width - SCROLLBAR_WIDTH));

    let rtl = Geometry::new(BOUNDS, 28.0, 100, true);
    assert!(rtl.lane().is_some_and(|lane| close(lane.x, BOUNDS.x)));
    assert!(close(rtl.rows_area().x, BOUNDS.x + SCROLLBAR_WIDTH));
}

#[test]
fn checkbox_and_label_mirror() {
    let start = CHECKLIST_PADDING_INLINE + CHECKBOX_SIZE + CHECK_LABEL_GAP;
    let ltr = Geometry::new(BOUNDS, 28.0, 3, false);
    let row = ltr.row_rect(1, 0.0);
    let check = ltr.checkbox_rect(row);
    assert!(close(check.x, BOUNDS.x + CHECKLIST_PADDING_INLINE));
    assert!(close(check.center_y(), row.center_y()));
    let lane = ltr.label_lane();
    assert!(close(lane.x, BOUNDS.x + start));
    assert!(close(lane.width, BOUNDS.width - start - CHECKLIST_PADDING_INLINE));

    let rtl = Geometry::new(BOUNDS, 28.0, 3, true);
    let check = rtl.checkbox_rect(rtl.row_rect(1, 0.0));
    assert!(close(check.x + check.width, BOUNDS.x + BOUNDS.width - CHECKLIST_PADDING_INLINE));
    let lane = rtl.label_lane();
    assert!(close(lane.x, BOUNDS.x + CHECKLIST_PADDING_INLINE));
    assert!(close(lane.x + lane.width, BOUNDS.x + BOUNDS.width - start));
}

#[test]
fn rows_follow_the_offset() {
    let geometry = Geometry::new(BOUNDS, 28.0, 100, false);
    assert!(close(geometry.row_rect(3, 0.0).y, BOUNDS.y + 84.0));
    assert!(close(geometry.row_rect(3, 50.0).y, BOUNDS.y + 34.0));
}

#[test]
fn offset_clamps_to_the_scroll_range() {
    let geometry = Geometry::new(BOUNDS, 28.0, 100, false);
    assert!(close(geometry.clamp(-5.0), 0.0));
    assert!(close(geometry.clamp(1.0e9), 2800.0 - BOUNDS.height));
    assert!(close(geometry.clamp(f32::NAN), 0.0));
}

#[test]
fn hit_tests_skip_the_lane() {
    let geometry = Geometry::new(BOUNDS, 28.0, 100, false);
    let in_lane = Point::new(BOUNDS.x + BOUNDS.width - 2.0, BOUNDS.y + 40.0);
    assert!(geometry.in_lane(in_lane));
    assert_eq!(geometry.row_at(in_lane, 0.0), None);
    let on_row = Point::new(BOUNDS.x + 40.0, BOUNDS.y + 42.0);
    assert!(!geometry.in_lane(on_row));
    assert_eq!(geometry.row_at(on_row, 0.0), Some(1));
    assert_eq!(geometry.row_at(Point::new(0.0, 0.0), 0.0), None);
}

#[test]
fn clip_outside_the_bounds_shows_no_rows() {
    let geometry = Geometry::new(BOUNDS, 28.0, 100, false);
    let above = Rectangle { x: 0.0, y: 0.0, width: 400.0, height: 10.0 };
    assert!(geometry.visible(0.0, above).is_empty());
}

#[test]
fn navigation_moves_and_clamps() {
    assert_eq!(navigate(2, Nav::Previous, 10, 6), Some(1));
    assert_eq!(navigate(0, Nav::Previous, 10, 6), Some(0));
    assert_eq!(navigate(8, Nav::Next, 10, 6), Some(9));
    assert_eq!(navigate(9, Nav::Next, 10, 6), Some(9));
    assert_eq!(navigate(1, Nav::PageDown, 10, 6), Some(7));
    assert_eq!(navigate(7, Nav::PageDown, 10, 6), Some(9));
    assert_eq!(navigate(3, Nav::PageUp, 10, 6), Some(0));
    assert_eq!(navigate(5, Nav::First, 10, 6), Some(0));
    assert_eq!(navigate(3, Nav::Last, 10, 6), Some(9));
    assert_eq!(navigate(20, Nav::Previous, 10, 6), Some(9));
    assert_eq!(navigate(0, Nav::Next, 0, 6), None);
}

#[test]
fn taps_stay_within_the_slop() {
    let start = Point::new(0.0, 0.0);
    assert!(is_tap(start, Point::new(3.0, 4.0)));
    assert!(is_tap(start, Point::new(0.0, TOUCH_SLOP)));
    assert!(!is_tap(start, Point::new(6.0, 8.0)));
}

#[test]
fn detects_right_to_left_script() {
    assert!(!has_rtl_script("Paris"));
    assert!(!has_rtl_script("Москва"));
    assert!(!has_rtl_script("서울"));
    assert!(has_rtl_script("باريس"));
    assert!(has_rtl_script("עברית"));
    assert!(has_rtl_script("Isfahan اصفهان"));
}

/// Simulator flows: rows are found through the labels `operate` reports.
#[cfg(not(target_family = "wasm"))]
mod simulator {
    use iced::keyboard::key::Named;
    use iced::widget::Id;
    use iced::{Element, Font};

    use super::Rows;
    use crate::app::widget::checklist::Checklist;

    #[derive(Clone, Debug, PartialEq)]
    enum TestMessage {
        Toggle(u32, bool),
    }

    fn list() -> Element<'static, TestMessage> {
        let rows = Rows(vec![("Alpha", false), ("Beta", true), ("Gamma", false)]);
        Checklist::new(rows)
            .id(Id::new("test.checklist"))
            .font(Font::DEFAULT)
            .on_toggle(TestMessage::Toggle)
            .into()
    }

    #[test]
    fn click_toggles_the_option_under_the_pointer() -> Result<(), iced_test::Error> {
        let mut ui = iced_test::simulator(list());
        let _ = ui.click("Beta")?;
        let messages: Vec<TestMessage> = ui.into_messages().collect();
        assert_eq!(messages, vec![TestMessage::Toggle(1, false)]);
        Ok(())
    }

    #[test]
    fn arrows_and_space_toggle_the_active_option() -> Result<(), iced_test::Error> {
        let mut ui = iced_test::simulator(list());
        let _ = ui.click("Alpha")?;
        let _ = ui.tap_key(Named::ArrowDown);
        let _ = ui.tap_key(Named::ArrowDown);
        let _ = ui.tap_key(Named::Space);
        let _ = ui.tap_key(Named::Home);
        let _ = ui.tap_key(Named::Space);
        let messages: Vec<TestMessage> = ui.into_messages().collect();
        assert_eq!(
            messages,
            vec![TestMessage::Toggle(0, true), TestMessage::Toggle(2, true), TestMessage::Toggle(0, true)]
        );
        Ok(())
    }

    #[test]
    fn keys_without_focus_do_nothing() {
        let mut ui = iced_test::simulator(list());
        let _ = ui.tap_key(Named::Space);
        assert_eq!(ui.into_messages().count(), 0);
    }
}
