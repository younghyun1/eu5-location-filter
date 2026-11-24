//! Chrome pass, drawn in its own layer above the column layers: header rule, column dividers,
//! the active column underline and focus ring, both scrollbar thumbs, and the empty state.

use iced::advanced::renderer::Renderer as _;
use iced::font::Weight;
use iced::{Border, Color, Rectangle};

use super::grid::ResultGrid;
use super::metrics::Metrics;
use super::paint::{self, Side, Style};
use super::state::{Axis, GridDrag, GridState};
use crate::app::layout::{
    ACTIVE_COLUMN_UNDERLINE, BORDER_WIDTH, FOCUS_RING_WIDTH, GRID_CELL_PADDING_INLINE,
    SCROLLBAR_THUMB_WIDTH, SCROLLBAR_THUMB_WIDTH_TOUCH, SPLITTER_ACTIVE_LINE_WIDTH,
};
use crate::app::theme::{TEXT_BODY, TEXT_TITLE, ThumbState, sans_for};
use crate::app::widget::window;

/// Space between the empty-state title and body.
const EMPTY_GAP: f32 = 8.0;

/// Everything drawn above cell text.
pub(super) fn chrome<Message>(
    grid: &ResultGrid<'_, Message>,
    state: &GridState,
    metrics: &Metrics,
    renderer: &mut iced::Renderer,
    clip: Rectangle,
) {
    let palette = grid.palette;
    let frame = &metrics.frame;
    let header = frame.header;
    let rule = Rectangle {
        y: header.y + header.height - BORDER_WIDTH,
        height: BORDER_WIDTH,
        ..header
    };
    paint::fill_rect(renderer, rule.intersection(&clip), palette.line_strong);
    dividers(grid, state, metrics, renderer, clip);
    if state.focused {
        if let Some(active) = grid.active_slot(state, &metrics.strip) {
            let underline = Rectangle {
                x: frame.screen_x(active.start, active.width, metrics.offset_x),
                y: header.y + header.height - ACTIVE_COLUMN_UNDERLINE,
                width: active.width,
                height: ACTIVE_COLUMN_UNDERLINE,
            };
            let visible = header
                .intersection(&clip)
                .and_then(|area| underline.intersection(&area));
            paint::fill_rect(renderer, visible, palette.accent);
        }
        if let Some(body) = frame.body.intersection(&clip) {
            let ring = Border {
                color: palette.accent,
                width: FOCUS_RING_WIDTH,
                radius: 0.0_f32.into(),
            };
            paint::fill_bordered(renderer, body, Color::TRANSPARENT, ring);
        }
    }
    thumbs(grid, state, metrics, renderer, clip);
    if metrics.rows == 0 {
        empty(grid, metrics, renderer, clip);
    }
}

/// A divider at every visible column's inline-end edge, through the header and, when rows
/// exist, the body; accent while its resize target is hovered or dragged.
fn dividers<Message>(
    grid: &ResultGrid<'_, Message>,
    state: &GridState,
    metrics: &Metrics,
    renderer: &mut iced::Renderer,
    clip: Rectangle,
) {
    let frame = &metrics.frame;
    let height = if metrics.rows == 0 {
        frame.header.height
    } else {
        frame.header.height + frame.body.height
    };
    let area = Rectangle {
        height,
        ..frame.header
    };
    let Some(area) = area.intersection(&clip) else {
        return;
    };
    let dragged = match state.drag {
        Some(GridDrag::Resize { column, .. }) => Some(column),
        _ => None,
    };
    for slot in metrics.strip.visible(metrics.offset_x, frame.body.width) {
        let active = dragged == Some(slot.id) || state.hovered_edge == Some(slot.id);
        let (width, color) = if active {
            (SPLITTER_ACTIVE_LINE_WIDTH, grid.palette.accent)
        } else {
            (BORDER_WIDTH, grid.palette.line)
        };
        let edge = slot.start + slot.width;
        let divider = Rectangle {
            x: frame.screen_x(edge - width, width, metrics.offset_x),
            width,
            ..area
        };
        paint::fill_rect(renderer, divider.intersection(&area), color);
    }
}

/// Thumb colour state for one axis.
fn thumb_state(state: &GridState, axis: Axis) -> ThumbState {
    match (state.drag, axis) {
        (Some(GridDrag::VerticalThumb { .. }), Axis::Vertical)
        | (Some(GridDrag::HorizontalThumb { .. }), Axis::Horizontal) => ThumbState::Dragged,
        _ if state.hovered_lane == Some(axis) => ThumbState::Hovered,
        _ => ThumbState::Idle,
    }
}

/// Both thumbs, centred in their lanes; none when the content fits.
fn thumbs<Message>(
    grid: &ResultGrid<'_, Message>,
    state: &GridState,
    metrics: &Metrics,
    renderer: &mut iced::Renderer,
    clip: Rectangle,
) {
    let frame = &metrics.frame;
    let breadth = if grid.touch {
        SCROLLBAR_THUMB_WIDTH_TOUCH
    } else {
        SCROLLBAR_THUMB_WIDTH
    };
    if let Some(thumb) = metrics.vertical_thumb(metrics.offset_y) {
        let lane = frame.vertical_lane;
        let width = breadth.min(lane.width);
        let bounds = Rectangle {
            x: lane.x + (lane.width - width) / 2.0,
            y: lane.y + thumb.start,
            width,
            height: thumb.length,
        };
        draw_thumb(
            renderer,
            bounds,
            clip,
            grid.palette
                .scrollbar_thumb(thumb_state(state, Axis::Vertical)),
        );
    }
    if let Some(thumb) = metrics.horizontal_thumb(metrics.offset_x) {
        let lane = frame.horizontal_lane;
        let height = breadth.min(lane.height);
        let bounds = Rectangle {
            x: window::mirror(frame.rtl, lane.x, lane.width, thumb.start, thumb.length),
            y: lane.y + (lane.height - height) / 2.0,
            width: thumb.length,
            height,
        };
        draw_thumb(
            renderer,
            bounds,
            clip,
            grid.palette
                .scrollbar_thumb(thumb_state(state, Axis::Horizontal)),
        );
    }
}

/// One rounded thumb when it is inside the clip; the chrome layer clips it.
fn draw_thumb(renderer: &mut iced::Renderer, bounds: Rectangle, clip: Rectangle, color: Color) {
    if bounds.intersection(&clip).is_none() {
        return;
    }
    let radius = bounds.width.min(bounds.height) / 2.0;
    let border = Border {
        color: Color::TRANSPARENT,
        width: 0.0,
        radius: radius.into(),
    };
    paint::fill_bordered(renderer, bounds, color, border);
}

/// Empty-state title and body, centred in the body.
fn empty<Message>(
    grid: &ResultGrid<'_, Message>,
    metrics: &Metrics,
    renderer: &mut iced::Renderer,
    clip: Rectangle,
) {
    let Some(body) = metrics.frame.body.intersection(&clip) else {
        return;
    };
    let locale = grid.context.locale;
    let palette = grid.palette;
    let full = metrics.frame.body;
    let title_style = Style {
        font: sans_for(locale, Weight::Semibold),
        size: TEXT_TITLE,
        color: palette.ink,
    };
    let body_style = Style {
        font: sans_for(locale, Weight::Normal),
        size: TEXT_BODY,
        color: palette.ink_muted,
    };
    let inset = GRID_CELL_PADDING_INLINE * 2.0;
    let width = (full.width - inset * 2.0).max(0.0);
    let title_height = metrics.row_height;
    let title = Rectangle {
        x: full.x + inset,
        y: full.center_y() - title_height,
        width,
        height: title_height,
    };
    renderer.with_layer(body, |renderer| {
        if !grid.empty_title.is_empty() {
            paint::fill_line(
                renderer,
                grid.empty_title,
                Side::Center,
                title,
                title_style,
                body,
            );
        }
        if !grid.empty_body.is_empty() && width > 0.0 {
            let top = title.y + title.height + EMPTY_GAP;
            paint::fill_block(
                renderer,
                grid.empty_body,
                full.center_x(),
                top,
                width,
                body_style,
                body,
            );
        }
    });
}
