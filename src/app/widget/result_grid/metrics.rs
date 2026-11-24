//! Per-event and per-frame metrics: regions, the column strip, clamped offsets, thumbs and hit
//! testing. `draw`, `update`, `operate` and `mouse_interaction` all build one of these from
//! the layout bounds, so they share one definition of where things are.

use iced::{Point, Rectangle};

use super::geometry::{Frame, Strip};
use super::grid::ResultGrid;
use super::state::GridState;
use crate::app::layout::{
    COLUMN_RESIZE_HIT_WIDTH, GRID_INLINE_INSET, GRID_ROW_HEIGHT, SCROLLBAR_MIN_THUMB,
};
use crate::app::widget::window::{self, Thumb};

/// What a point hits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Hit {
    /// Resize target of a strip position.
    Edge(usize),
    /// Header cell of a strip position.
    Header(usize),
    /// Body row.
    Row(usize),
    /// Vertical scrollbar lane.
    VerticalLane,
    /// Horizontal scrollbar lane.
    HorizontalLane,
}

/// Everything derived from the layout bounds and the borrowed data for one event or frame.
#[derive(Clone, Copy, Debug)]
pub(super) struct Metrics {
    /// Screen regions.
    pub(super) frame: Frame,
    /// Visible columns.
    pub(super) strip: Strip,
    /// Number of result rows.
    pub(super) rows: usize,
    /// Row height in use.
    pub(super) row_height: f32,
    /// Clamped horizontal offset.
    pub(super) offset_x: f32,
    /// Clamped vertical offset.
    pub(super) offset_y: f32,
}

impl Metrics {
    /// Metrics for a grid at some bounds with the offsets of `state` clamped to the content.
    pub(super) fn new<Message>(
        grid: &ResultGrid<'_, Message>,
        bounds: Rectangle,
        state: &GridState,
    ) -> Self {
        let row_height = if grid.row_height.is_finite() && grid.row_height >= 1.0 {
            grid.row_height
        } else {
            GRID_ROW_HEIGHT
        };
        let frame = Frame::new(bounds, grid.direction);
        let strip = Strip::new(grid.columns);
        let mut metrics = Self {
            frame,
            strip,
            rows: grid.ids.len(),
            row_height,
            offset_x: 0.0,
            offset_y: 0.0,
        };
        metrics.offset_x = metrics.clamp_x(state.offset_x);
        metrics.offset_y = metrics.clamp_y(state.offset_y);
        metrics
    }

    /// Total body content height.
    pub(super) fn content_height(&self) -> f32 {
        self.rows as f32 * self.row_height
    }

    /// Horizontal offset clamped to the content.
    pub(super) fn clamp_x(&self, offset: f32) -> f32 {
        window::clamp_offset(offset, self.strip.content_width(), self.frame.body.width)
    }

    /// Vertical offset clamped to the content.
    pub(super) fn clamp_y(&self, offset: f32) -> f32 {
        window::clamp_offset(offset, self.content_height(), self.frame.body.height)
    }

    /// Rows inside the body viewport.
    pub(super) fn visible_rows(&self) -> window::VisibleRange {
        window::visible_rows(
            self.offset_y,
            self.frame.body.height,
            self.row_height,
            self.rows,
        )
    }

    /// Screen y of the top of a row.
    pub(super) fn row_y(&self, row: usize) -> f32 {
        self.frame.body.y + row as f32 * self.row_height - self.offset_y
    }

    /// Vertical thumb along the body height.
    pub(super) fn vertical_thumb(&self, offset_y: f32) -> Option<Thumb> {
        let body = self.frame.body;
        window::thumb(
            offset_y,
            self.content_height(),
            body.height,
            body.height,
            SCROLLBAR_MIN_THUMB,
        )
    }

    /// Horizontal thumb along the body width, measured from the inline start of the lane.
    pub(super) fn horizontal_thumb(&self, offset_x: f32) -> Option<Thumb> {
        let width = self.frame.body.width;
        window::thumb(
            offset_x,
            self.strip.content_width(),
            width,
            width,
            SCROLLBAR_MIN_THUMB,
        )
    }

    /// Hit test of a screen point.
    pub(super) fn hit(&self, point: Point) -> Option<Hit> {
        let frame = &self.frame;
        if frame.header.contains(point) {
            let inline = frame.inline_x(point.x, self.offset_x) - GRID_INLINE_INSET;
            let widths = self.strip.widths();
            if let Some(index) = window::edge_at(inline, widths, COLUMN_RESIZE_HIT_WIDTH) {
                return Some(Hit::Edge(index));
            }
            return window::column_at(inline, widths).map(Hit::Header);
        }
        if frame.body.contains(point) {
            let y = point.y - frame.body.y;
            return window::row_at(y, self.offset_y, self.row_height, self.rows).map(Hit::Row);
        }
        if frame.vertical_lane.contains(point) {
            return Some(Hit::VerticalLane);
        }
        if frame.horizontal_lane.contains(point) {
            return Some(Hit::HorizontalLane);
        }
        None
    }
}
