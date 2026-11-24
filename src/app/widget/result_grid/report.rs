//! What the grid reports to widget operations besides focus: the visible header titles and
//! visible cell texts with their on-screen bounds, so simulator selectors can find and click
//! them. Like drawing, it touches only visible rows and columns.

use iced::Rectangle;
use iced::advanced::widget::Operation;

use super::grid::ResultGrid;
use super::metrics::Metrics;
use super::state::GridState;
use crate::app::i18n::t;
use crate::app::view_model::write_cell;

/// Reports visible header titles, then visible cells row by row.
pub(super) fn visible_text<Message>(
    grid: &ResultGrid<'_, Message>,
    state: &mut GridState,
    metrics: &Metrics,
    operation: &mut dyn Operation,
) {
    let frame = &metrics.frame;
    let locale = grid.context.locale;
    for slot in metrics.strip.visible(metrics.offset_x, frame.body.width) {
        let cell = Rectangle {
            x: frame.screen_x(slot.start, slot.width, metrics.offset_x),
            width: slot.width,
            ..frame.header
        };
        if let Some(bounds) = cell.intersection(&frame.header) {
            operation.text(None, bounds, t(locale, slot.id.title()));
        }
    }
    let rows = metrics.visible_rows();
    if rows.is_empty() {
        return;
    }
    let dataset = grid.context.labels.dataset();
    let buffer = state.buffer.get_mut();
    for row in rows.start..rows.end {
        let Some(record) = grid.ids.get(row).and_then(|id| dataset.location(*id)) else {
            continue;
        };
        for slot in metrics.strip.visible(metrics.offset_x, frame.body.width) {
            let cell = Rectangle {
                x: frame.screen_x(slot.start, slot.width, metrics.offset_x),
                y: metrics.row_y(row),
                width: slot.width,
                height: metrics.row_height,
            };
            let Some(bounds) = cell.intersection(&frame.body) else {
                continue;
            };
            write_cell(buffer, &grid.context, record, slot.id);
            if !buffer.is_empty() {
                operation.text(None, bounds, buffer.as_str());
            }
        }
    }
}
