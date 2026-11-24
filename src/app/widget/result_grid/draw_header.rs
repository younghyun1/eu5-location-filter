//! Header pass: one clipping layer per visible column holding its title and, for the sorted
//! column, the sort arrow in the slot reserved at the inline end.

use iced::Rectangle;
use iced::advanced::renderer::Renderer as _;
use iced::font::Weight;

use super::grid::ResultGrid;
use super::metrics::Metrics;
use super::paint::{self, Side, Style};
use crate::app::i18n::t;
use crate::app::layout::{GRID_CELL_PADDING_INLINE, SORT_GLYPH_WIDTH};
use crate::app::theme::{Icon, TEXT_SMALL, sans_for};

/// Column titles and the sort arrow.
pub(super) fn labels<Message>(
    grid: &ResultGrid<'_, Message>,
    metrics: &Metrics,
    renderer: &mut iced::Renderer,
    clip: Rectangle,
) {
    let frame = &metrics.frame;
    let Some(header) = frame.header.intersection(&clip) else {
        return;
    };
    let palette = grid.palette;
    let locale = grid.context.locale;
    let rtl = frame.rtl;
    let title_font = sans_for(locale, Weight::Medium);
    for slot in metrics.strip.visible(metrics.offset_x, frame.body.width) {
        let cell = Rectangle {
            x: frame.screen_x(slot.start, slot.width, metrics.offset_x),
            width: slot.width,
            ..frame.header
        };
        let Some(layer) = cell.intersection(&header) else {
            continue;
        };
        let sorted = slot.id.sort_field() == grid.sort.field;
        let inner_left = cell.x + GRID_CELL_PADDING_INLINE;
        let inner_right = cell.x + cell.width - GRID_CELL_PADDING_INLINE;
        let (title_left, title_right, glyph_left) = if rtl {
            (inner_left + SORT_GLYPH_WIDTH, inner_right, inner_left)
        } else {
            (
                inner_left,
                inner_right - SORT_GLYPH_WIDTH,
                inner_right - SORT_GLYPH_WIDTH,
            )
        };
        renderer.with_layer(layer, |renderer| {
            if title_right > title_left {
                let style = Style {
                    font: title_font,
                    size: TEXT_SMALL,
                    color: if sorted {
                        palette.ink
                    } else {
                        palette.ink_muted
                    },
                };
                let area = Rectangle {
                    x: title_left,
                    width: title_right - title_left,
                    ..cell
                };
                let title = t(locale, slot.id.title());
                paint::fill_line(
                    renderer,
                    title,
                    Side::of(slot.id.align(), rtl),
                    area,
                    style,
                    layer,
                );
            }
            if sorted {
                let icon = if grid.sort.ascending {
                    Icon::SortAscending
                } else {
                    Icon::SortDescending
                };
                let mut utf8 = [0_u8; 4];
                let glyph = icon.glyph(grid.direction).encode_utf8(&mut utf8);
                let style = Style {
                    font: icon.font(),
                    size: TEXT_SMALL,
                    color: palette.ink,
                };
                let area = Rectangle {
                    x: glyph_left,
                    width: SORT_GLYPH_WIDTH,
                    ..cell
                };
                paint::fill_line(renderer, glyph, Side::Center, area, style, layer);
            }
        });
    }
}
