//! Pure geometry and decisions of the checklist: row and checkbox rectangles in screen space,
//! the scrollbar lane and thumb, key navigation and the tap test. Everything here is free of
//! renderer and shell types so it can be unit tested; screen positions go through
//! `window::mirror` so right-to-left layouts reflect the same inline geometry. Key navigation,
//! the tap test and right-to-left label detection live in the `rules` child module; row height
//! helpers live in `metrics`.

use iced::{Point, Rectangle};

use crate::app::layout::{
    CHECK_LABEL_GAP, CHECKBOX_SIZE, CHECKLIST_PADDING_INLINE, SCROLLBAR_MIN_THUMB,
    SCROLLBAR_THUMB_WIDTH, SCROLLBAR_THUMB_WIDTH_TOUCH, SCROLLBAR_WIDTH,
};
use crate::app::widget::window::{self, Thumb, VisibleRange};

mod metrics;
mod rules;

pub(super) use metrics::{list_height, rows_to_pixels, sane_row_height};
pub(super) use rules::{Nav, has_rtl_script, is_tap, navigate};

/// Geometry of one checklist for one frame or event.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Geometry {
    /// Widget bounds; also the scroll viewport.
    pub(super) bounds: Rectangle,
    /// Height of every row.
    pub(super) row_height: f32,
    /// Number of rows in the model.
    pub(super) count: usize,
    /// Right-to-left layout: checkboxes at the right, scrollbar at the left.
    pub(super) rtl: bool,
}

impl Geometry {
    /// Geometry for the given bounds, metrics and row count.
    #[must_use]
    pub(super) fn new(bounds: Rectangle, row_height: f32, count: usize, rtl: bool) -> Self {
        Self {
            bounds,
            row_height: sane_row_height(row_height),
            count,
            rtl,
        }
    }

    /// Total height of all rows.
    #[must_use]
    pub(super) fn content_height(&self) -> f32 {
        rows_to_pixels(self.count, self.row_height)
    }

    /// Whether the rows overflow the bounds, which adds the scrollbar lane.
    #[must_use]
    pub(super) fn is_scrollable(&self) -> bool {
        window::max_offset(self.content_height(), self.bounds.height) > 0.0
    }

    /// Offset clamped to the scroll range of this geometry.
    #[must_use]
    pub(super) fn clamp(&self, offset: f32) -> f32 {
        window::clamp_offset(offset, self.content_height(), self.bounds.height)
    }

    /// Width of the scrollbar lane, zero when the content fits.
    #[must_use]
    fn lane_width(&self) -> f32 {
        if self.is_scrollable() {
            SCROLLBAR_WIDTH.min(self.bounds.width)
        } else {
            0.0
        }
    }

    /// The scrollbar lane at the inline end, when the content overflows.
    #[must_use]
    pub(super) fn lane(&self) -> Option<Rectangle> {
        let width = self.lane_width();
        if width <= 0.0 {
            return None;
        }
        let x = window::mirror(
            self.rtl,
            self.bounds.x,
            self.bounds.width,
            self.bounds.width - width,
            width,
        );
        Some(Rectangle {
            x,
            y: self.bounds.y,
            width,
            height: self.bounds.height,
        })
    }

    /// Area the rows occupy: the bounds without the scrollbar lane.
    #[must_use]
    pub(super) fn rows_area(&self) -> Rectangle {
        let width = (self.bounds.width - self.lane_width()).max(0.0);
        let x = window::mirror(self.rtl, self.bounds.x, self.bounds.width, 0.0, width);
        Rectangle {
            x,
            y: self.bounds.y,
            width,
            height: self.bounds.height,
        }
    }

    /// Rows intersecting the part of the bounds that is inside `clip`, so a list inside an
    /// outer scrollable draws only what that scrollable shows.
    #[must_use]
    pub(super) fn visible(&self, offset: f32, clip: Rectangle) -> VisibleRange {
        let top = (clip.y - self.bounds.y).max(0.0);
        let bottom = (clip.y + clip.height - self.bounds.y).min(self.bounds.height);
        if bottom <= top {
            return VisibleRange::default();
        }
        window::visible_rows(offset + top, bottom - top, self.row_height, self.count)
    }

    /// Screen rectangle of a row across the rows area.
    #[must_use]
    pub(super) fn row_rect(&self, row: usize, offset: f32) -> Rectangle {
        let area = self.rows_area();
        Rectangle {
            x: area.x,
            y: self.bounds.y + rows_to_pixels(row, self.row_height) - offset,
            width: area.width,
            height: self.row_height,
        }
    }

    /// Checkbox square inside a row, at the inline start after the row padding.
    #[must_use]
    pub(super) fn checkbox_rect(&self, row: Rectangle) -> Rectangle {
        let area = self.rows_area();
        let x = window::mirror(
            self.rtl,
            area.x,
            area.width,
            CHECKLIST_PADDING_INLINE,
            CHECKBOX_SIZE,
        );
        Rectangle {
            x,
            y: row.y + (row.height - CHECKBOX_SIZE) / 2.0,
            width: CHECKBOX_SIZE,
            height: CHECKBOX_SIZE,
        }
    }

    /// Strip holding every label: from after the checkbox gap to the inline-end padding, over
    /// the full bounds height. One clip layer over this strip clips every label.
    #[must_use]
    pub(super) fn label_lane(&self) -> Rectangle {
        let area = self.rows_area();
        let start = CHECKLIST_PADDING_INLINE + CHECKBOX_SIZE + CHECK_LABEL_GAP;
        let width = (area.width - start - CHECKLIST_PADDING_INLINE).max(0.0);
        let x = window::mirror(self.rtl, area.x, area.width, start, width);
        Rectangle {
            x,
            y: self.bounds.y,
            width,
            height: self.bounds.height,
        }
    }

    /// Row under an absolute point, excluding the scrollbar lane.
    #[must_use]
    pub(super) fn row_at(&self, point: Point, offset: f32) -> Option<usize> {
        if !self.rows_area().contains(point) {
            return None;
        }
        window::row_at(point.y - self.bounds.y, offset, self.row_height, self.count)
    }

    /// Whether an absolute point is over the scrollbar lane.
    #[must_use]
    pub(super) fn in_lane(&self, point: Point) -> bool {
        self.lane().is_some_and(|lane| lane.contains(point))
    }

    /// Thumb along the lane, when the content overflows.
    #[must_use]
    pub(super) fn thumb(&self, offset: f32) -> Option<Thumb> {
        if !self.is_scrollable() {
            return None;
        }
        let height = self.bounds.height;
        window::thumb(
            offset,
            self.content_height(),
            height,
            height,
            SCROLLBAR_MIN_THUMB,
        )
    }

    /// Drawn thumb rectangle, centred across the lane.
    #[must_use]
    pub(super) fn thumb_rect(&self, offset: f32, touch: bool) -> Option<Rectangle> {
        let lane = self.lane()?;
        let thumb = self.thumb(offset)?;
        let width = if touch {
            SCROLLBAR_THUMB_WIDTH_TOUCH
        } else {
            SCROLLBAR_THUMB_WIDTH
        };
        let width = width.min(lane.width);
        Some(Rectangle {
            x: lane.x + (lane.width - width) / 2.0,
            y: lane.y + thumb.start,
            width,
            height: thumb.length,
        })
    }

    /// Offset for a thumb whose start sits at `thumb_start` pixels from the lane top.
    #[must_use]
    pub(super) fn offset_for_thumb(&self, thumb_start: f32) -> f32 {
        let height = self.bounds.height;
        let offset = window::offset_for_thumb(
            thumb_start,
            self.content_height(),
            height,
            height,
            SCROLLBAR_MIN_THUMB,
        );
        self.clamp(offset)
    }

    /// First row whose top is inside the viewport, the active row a fresh focus starts from.
    #[must_use]
    pub(super) fn first_visible(&self, offset: f32) -> Option<usize> {
        let range = window::visible_rows(offset, self.bounds.height, self.row_height, self.count);
        if range.is_empty() {
            return if self.count > 0 { Some(0) } else { None };
        }
        let top = rows_to_pixels(range.start, self.row_height);
        let first = if top < offset {
            range.start + 1
        } else {
            range.start
        };
        Some(first.min(self.count.saturating_sub(1)))
    }

    /// Offset that shows `row` fully, clamped to the scroll range.
    #[must_use]
    pub(super) fn reveal(&self, offset: f32, row: usize) -> f32 {
        self.clamp(window::scroll_into_view(
            offset,
            row,
            self.row_height,
            self.bounds.height,
        ))
    }

    /// Rows a page key moves.
    #[must_use]
    pub(super) fn page(&self) -> usize {
        window::page_rows(self.bounds.height, self.row_height).max(1)
    }
}
