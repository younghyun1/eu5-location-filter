//! Screen regions of the grid, the visible column strip, and the mapping between screen x and
//! inline content x. Everything is computed on the stack from the layout bounds.

use iced::Rectangle;

use crate::app::layout::{Direction, GRID_HEADER_HEIGHT, GRID_INLINE_INSET, SCROLLBAR_WIDTH};
use crate::app::view_model::{ColumnId, Columns};
use crate::app::widget::window;

/// Visible columns in display order with their inline starts, held in fixed arrays sized by
/// the column count.
#[derive(Clone, Copy, Debug)]
pub(super) struct Strip {
    ids: [ColumnId; ColumnId::COUNT],
    widths: [f32; ColumnId::COUNT],
    starts: [f32; ColumnId::COUNT],
    len: usize,
    content_width: f32,
}

/// One visible column.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Slot {
    /// Position in the strip.
    pub(super) index: usize,
    /// Column identity.
    pub(super) id: ColumnId,
    /// Inline start from the content origin, the inset included.
    pub(super) start: f32,
    /// Width.
    pub(super) width: f32,
}

impl Strip {
    /// Visible columns of `columns`; non-finite or negative widths count as zero.
    pub(super) fn new(columns: &Columns) -> Self {
        let mut strip = Self {
            ids: [ColumnId::Color; ColumnId::COUNT],
            widths: [0.0; ColumnId::COUNT],
            starts: [0.0; ColumnId::COUNT],
            len: 0,
            content_width: GRID_INLINE_INSET,
        };
        for column in columns.visible() {
            let width = if column.width.is_finite() {
                column.width.max(0.0)
            } else {
                0.0
            };
            let index = strip.len;
            if let (Some(id), Some(slot_width), Some(start)) = (
                strip.ids.get_mut(index),
                strip.widths.get_mut(index),
                strip.starts.get_mut(index),
            ) {
                *id = column.id;
                *slot_width = width;
                *start = strip.content_width;
                strip.content_width += width;
                strip.len += 1;
            }
        }
        strip
    }

    /// Number of visible columns.
    pub(super) const fn len(&self) -> usize {
        self.len
    }

    /// Visible widths in display order.
    pub(super) fn widths(&self) -> &[f32] {
        match self.widths.get(..self.len) {
            Some(widths) => widths,
            None => &[],
        }
    }

    /// Content width: inset plus every visible width.
    pub(super) const fn content_width(&self) -> f32 {
        self.content_width
    }

    /// The column at a strip position.
    pub(super) fn slot(&self, index: usize) -> Option<Slot> {
        if index >= self.len {
            return None;
        }
        match (
            self.ids.get(index),
            self.starts.get(index),
            self.widths.get(index),
        ) {
            (Some(id), Some(start), Some(width)) => Some(Slot {
                index,
                id: *id,
                start: *start,
                width: *width,
            }),
            _ => None,
        }
    }

    /// Strip position of a column, or `None` when it is hidden.
    pub(super) fn position(&self, id: ColumnId) -> Option<usize> {
        self.ids
            .get(..self.len)?
            .iter()
            .position(|candidate| *candidate == id)
    }

    /// Columns intersecting the inline window `offset..offset + extent`.
    pub(super) fn visible(&self, offset: f32, extent: f32) -> impl Iterator<Item = Slot> + '_ {
        (0..self.len)
            .filter_map(|index| self.slot(index))
            .filter(move |slot| slot.start < offset + extent && slot.start + slot.width > offset)
    }
}

/// Screen regions. Both scrollbar lanes are always reserved, so the body viewport does not
/// depend on whether the content overflows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Frame {
    /// Whole widget.
    pub(super) bounds: Rectangle,
    /// Sticky header row above the body, as wide as the body.
    pub(super) header: Rectangle,
    /// Row viewport.
    pub(super) body: Rectangle,
    /// Vertical scrollbar lane at the inline end of the body.
    pub(super) vertical_lane: Rectangle,
    /// Horizontal scrollbar lane under the body.
    pub(super) horizontal_lane: Rectangle,
    /// Mirrored layout.
    pub(super) rtl: bool,
}

impl Frame {
    /// Regions for widget bounds in a direction.
    pub(super) fn new(bounds: Rectangle, direction: Direction) -> Self {
        let rtl = direction.is_rtl();
        let width = bounds.width.max(0.0);
        let height = bounds.height.max(0.0);
        let lane = SCROLLBAR_WIDTH.min(width);
        let view_width = width - lane;
        let view_x = if rtl { bounds.x + lane } else { bounds.x };
        let header_height = GRID_HEADER_HEIGHT.min(height);
        let lane_height = SCROLLBAR_WIDTH.min(height - header_height);
        let body_height = height - header_height - lane_height;
        let body_y = bounds.y + header_height;
        Self {
            bounds,
            header: Rectangle {
                x: view_x,
                y: bounds.y,
                width: view_width,
                height: header_height,
            },
            body: Rectangle {
                x: view_x,
                y: body_y,
                width: view_width,
                height: body_height,
            },
            vertical_lane: Rectangle {
                x: if rtl { bounds.x } else { bounds.x + view_width },
                y: body_y,
                width: lane,
                height: body_height,
            },
            horizontal_lane: Rectangle {
                x: view_x,
                y: body_y + body_height,
                width: view_width,
                height: lane_height,
            },
            rtl,
        }
    }

    /// Screen x of the left edge of an inline content span at a horizontal offset.
    pub(super) fn screen_x(&self, inline_x: f32, width: f32, offset_x: f32) -> f32 {
        window::mirror(
            self.rtl,
            self.body.x,
            self.body.width,
            inline_x - offset_x,
            width,
        )
    }

    /// Inline content x under a screen x at a horizontal offset.
    pub(super) fn inline_x(&self, screen_x: f32, offset_x: f32) -> f32 {
        if self.rtl {
            self.body.x + self.body.width - screen_x + offset_x
        } else {
            screen_x - self.body.x + offset_x
        }
    }

    /// Screen delta to inline delta (positive toward the inline end).
    pub(super) const fn to_inline(self, screen_dx: f32) -> f32 {
        if self.rtl { -screen_dx } else { screen_dx }
    }

    /// Inline position of a screen x inside the horizontal lane, measured from its inline start.
    pub(super) fn lane_inline(&self, screen_x: f32) -> f32 {
        let lane = self.horizontal_lane;
        if self.rtl {
            lane.x + lane.width - screen_x
        } else {
            screen_x - lane.x
        }
    }
}
