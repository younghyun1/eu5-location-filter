//! Body pass: row grounds in the parent layer, then one clipping layer per visible column
//! holding that column's swatches and text for every visible row. Only visible rows and
//! visible columns are touched, and cell text is formatted into the one reused buffer.

use iced::advanced::renderer::Renderer as _;
use iced::font::Weight;
use iced::{Border, Color, Rectangle};

use super::grid::ResultGrid;
use super::metrics::Metrics;
use super::paint::{self, Side, Style};
use super::state::GridState;
use crate::app::layout::{
    BORDER_WIDTH, GRID_CELL_PADDING_INLINE, RADIUS, SELECTED_BAR_WIDTH, SWATCH_GAP, SWATCH_SIZE,
};
use crate::app::theme::{MONO, TEXT_BODY, TEXT_META, sans_for};
use crate::app::view_model::{CellFont, ColumnId, tone_color, write_cell};
use crate::model::{LocationRecord, MapColor};

/// Widget ground, header ground, zebra, hover and selection grounds, and the selected-row bar.
pub(super) fn grounds<Message>(
    grid: &ResultGrid<'_, Message>,
    state: &GridState,
    metrics: &Metrics,
    renderer: &mut iced::Renderer,
    clip: Rectangle,
) {
    let palette = grid.palette;
    let frame = &metrics.frame;
    paint::fill_rect(renderer, frame.bounds.intersection(&clip), palette.surface);
    paint::fill_rect(renderer, frame.header.intersection(&clip), palette.surface_2());
    let Some(body) = frame.body.intersection(&clip) else {
        return;
    };
    let rows = metrics.visible_rows();
    for row in rows.start..rows.end {
        let selected = grid.selected.is_some() && grid.ids.get(row).copied() == grid.selected;
        let base = if selected {
            palette.accent_soft
        } else if row % 2 == 1 {
            palette.zebra()
        } else {
            palette.surface
        };
        let ground = if !selected && state.hovered_row == Some(row) {
            palette.hover(base)
        } else {
            base
        };
        let rect = Rectangle {
            x: frame.body.x,
            y: metrics.row_y(row),
            width: frame.body.width,
            height: metrics.row_height,
        };
        if ground != palette.surface {
            paint::fill_rect(renderer, rect.intersection(&body), ground);
        }
        if selected {
            let bar = Rectangle {
                x: frame.screen_x(0.0, SELECTED_BAR_WIDTH, metrics.offset_x),
                width: SELECTED_BAR_WIDTH,
                ..rect
            };
            paint::fill_rect(renderer, bar.intersection(&body), palette.accent);
        }
    }
}

/// Cell swatches and text, one layer per visible column so text clips on every backend.
pub(super) fn cells<Message>(
    grid: &ResultGrid<'_, Message>,
    metrics: &Metrics,
    renderer: &mut iced::Renderer,
    clip: Rectangle,
    buffer: &mut String,
) {
    let frame = &metrics.frame;
    let Some(body) = frame.body.intersection(&clip) else {
        return;
    };
    let rows = metrics.visible_rows();
    if rows.is_empty() {
        return;
    }
    let dataset = grid.context.labels.dataset();
    for slot in metrics.strip.visible(metrics.offset_x, frame.body.width) {
        let x = frame.screen_x(slot.start, slot.width, metrics.offset_x);
        let column = Rectangle { x, width: slot.width, ..body };
        let Some(layer) = column.intersection(&body) else {
            continue;
        };
        renderer.with_layer(layer, |renderer| {
            for row in rows.start..rows.end {
                let Some(record) = grid.ids.get(row).and_then(|id| dataset.location(*id)) else {
                    continue;
                };
                let cell = Rectangle {
                    x,
                    y: metrics.row_y(row),
                    width: slot.width,
                    height: metrics.row_height,
                };
                let Some(cell_clip) = cell.intersection(&layer) else {
                    continue;
                };
                draw_cell(grid, renderer, buffer, record, slot.id, cell, cell_clip);
            }
        });
    }
}

/// One cell: the swatch for the colour column, then the text against the cell's logical side.
fn draw_cell<Message>(
    grid: &ResultGrid<'_, Message>,
    renderer: &mut iced::Renderer,
    buffer: &mut String,
    record: &LocationRecord,
    column: ColumnId,
    cell: Rectangle,
    clip: Rectangle,
) {
    let palette = grid.palette;
    let rtl = grid.direction.is_rtl();
    let tone = write_cell(buffer, &grid.context, record, column);
    let mut left = cell.x + GRID_CELL_PADDING_INLINE;
    let mut right = cell.x + cell.width - GRID_CELL_PADDING_INLINE;
    let font = column.font();
    if font == CellFont::Swatch {
        let swatch = Rectangle {
            x: if rtl { right - SWATCH_SIZE } else { left },
            y: cell.center_y() - SWATCH_SIZE / 2.0,
            width: SWATCH_SIZE,
            height: SWATCH_SIZE,
        };
        let border = Border {
            color: palette.line_strong,
            width: BORDER_WIDTH,
            radius: RADIUS.into(),
        };
        paint::fill_bordered(renderer, swatch, swatch_color(record.color), border);
        let used = SWATCH_SIZE + SWATCH_GAP;
        if rtl {
            right -= used;
        } else {
            left += used;
        }
    }
    if right <= left || buffer.is_empty() {
        return;
    }
    let locale = grid.context.locale;
    let (face, size) = match font {
        CellFont::Sans => (sans_for(locale, Weight::Normal), TEXT_BODY),
        CellFont::SansMedium => (sans_for(locale, Weight::Medium), TEXT_BODY),
        CellFont::Mono | CellFont::Swatch => (MONO, TEXT_META),
    };
    let style = Style {
        font: face,
        size,
        color: tone_color(palette, tone),
    };
    let area = Rectangle {
        x: left,
        width: right - left,
        ..cell
    };
    paint::fill_line(renderer, buffer.as_str(), Side::of(column.align(), rtl), area, style, clip);
}

/// Opaque swatch colour of a packed `0xRRGGBB` map colour; the masks keep each channel in
/// `u8` range, so the casts are lossless.
fn swatch_color(color: MapColor) -> Color {
    let value = color.0;
    Color::from_rgb8(
        ((value >> 16) & 0xFF) as u8,
        ((value >> 8) & 0xFF) as u8,
        (value & 0xFF) as u8,
    )
}
