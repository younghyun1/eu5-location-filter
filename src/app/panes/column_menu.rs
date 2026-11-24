//! Column menu popover: a checklist of all columns without a search field. Shows
//! `column_menu.keep_one` while only one column is visible.

use iced::font::Weight;
use iced::widget::{Column, container, space};
use iced::{Element, Length, Theme};

use crate::app::context::ViewContext;
use crate::app::i18n::Text;
use crate::app::ids;
use crate::app::layout::{
    CHECKLIST_ROW_HEIGHT, CHECKLIST_ROW_HEIGHT_TOUCH, LEGEND_MARK_SIZE, MENU_MAX_HEIGHT,
    MENU_PADDING, SWATCH_GAP, TOUCH_TARGET, inline_row,
};
use crate::app::message::{ColumnMessage, Message};
use crate::app::panes::controls::menu_frame;
use crate::app::panes::overlays::kit::{mono, open_surface, prose};
use crate::app::theme::{self, TEXT_CAPTION, TEXT_SMALL, sans_for};
use crate::app::view_model::{ColumnId, ColumnRows, Columns};
use crate::app::widget::Checklist;

/// Height the menu spends on everything but the list: the title row, the paddings between the
/// blocks and room for the hint and, in the narrow layout, the legend.
const CHROME_ALLOWANCE: f32 = TOUCH_TARGET * 3.0 + MENU_PADDING * 6.0;

/// Rows the checklist shows before it scrolls itself, so the whole menu stays within
/// `MENU_MAX_HEIGHT`; the checklist owns the scrolling and its keyboard handling.
#[must_use]
pub(super) fn visible_rows(row_height: f32) -> usize {
    let rows = ((MENU_MAX_HEIGHT - CHROME_ALLOWANCE) / row_height).floor();
    if rows.is_finite() && rows >= 1.0 {
        // Positive and far below `usize::MAX`, so the conversion is exact.
        rows as usize
    } else {
        1
    }
}

/// Column of a checklist option index, or `None` for an index outside the column list.
#[must_use]
pub(super) fn column_for_option(option: u32) -> Option<ColumnId> {
    usize::try_from(option)
        .ok()
        .and_then(|index| ColumnId::ALL.get(index))
        .copied()
}

/// Menu content inside the shared menu frame; toggles publish `ColumnMessage::SetVisible`.
#[must_use]
pub(crate) fn view<'a>(ctx: ViewContext, columns: &'a Columns) -> Element<'a, Message> {
    let palette = ctx.palette;
    let narrow = ctx.is_narrow();
    let row_height = if narrow {
        CHECKLIST_ROW_HEIGHT_TOUCH
    } else {
        CHECKLIST_ROW_HEIGHT
    };
    let list = Checklist::new(ColumnRows {
        columns,
        locale: ctx.locale,
    })
    .id(ids::COLUMN_MENU_LIST)
    .direction(ctx.direction)
    .palette(palette)
    .font(sans_for(ctx.locale, Weight::Normal))
    .row_height(row_height)
    .max_visible_rows(visible_rows(row_height))
    .touch(narrow)
    .on_toggle(move |option, visible| match column_for_option(option) {
        Some(column) => Message::Column(ColumnMessage::SetVisible { column, visible }),
        // An index outside the list cannot come from the rows shown; restating the current
        // visibility of the first column keeps the callback total and changes nothing.
        None => Message::Column(ColumnMessage::SetVisible {
            column: ColumnId::Color,
            visible: columns.get(ColumnId::Color).visible,
        }),
    });
    let mut body = Column::new()
        .push(list)
        .spacing(MENU_PADDING)
        .width(Length::Fill);
    if columns.visible_count() <= 1 {
        body = body.push(prose(
            ctx,
            ctx.t(Text::ColumnMenuKeepOne),
            TEXT_SMALL,
            palette.ink_muted,
        ));
    }
    if narrow {
        body = body.push(legend(ctx));
    }
    open_surface(menu_frame(
        ctx,
        ctx.t(Text::ColumnMenuTitle),
        ctx.t(Text::ColumnMenuClose),
        body.into(),
    ))
}

/// Food and precious legend. The narrow toolbar has no room for it, so it ends the menu.
fn legend<'a>(ctx: ViewContext) -> Element<'a, Message> {
    let palette = ctx.palette;
    let entry = |color, label: &'static str| -> Element<'a, Message> {
        let mark = container(space())
            .width(LEGEND_MARK_SIZE)
            .height(LEGEND_MARK_SIZE)
            .style(move |_: &Theme| theme::swatch(palette, color));
        inline_row(
            ctx.direction,
            vec![
                mark.into(),
                mono(ctx, label, TEXT_CAPTION, true, palette.ink_muted).into(),
            ],
        )
        .spacing(SWATCH_GAP)
        .align_y(iced::alignment::Vertical::Center)
        .into()
    };
    inline_row(
        ctx.direction,
        vec![
            entry(palette.food(), ctx.t(Text::GridLegendFood)),
            entry(palette.precious(), ctx.t(Text::GridLegendPrecious)),
        ],
    )
    .spacing(MENU_PADDING * 2.0)
    .into()
}

#[cfg(test)]
mod tests {
    use super::{column_for_option, view, visible_rows};
    use crate::app::layout::{
        CHECKLIST_ROW_HEIGHT, CHECKLIST_ROW_HEIGHT_TOUCH, MENU_MAX_HEIGHT,
    };
    use crate::app::panes::overlays::kit::test_support::contexts;
    use crate::app::view_model::{ColumnId, Columns};

    #[test]
    fn options_map_to_columns_in_display_order() {
        for (index, column) in ColumnId::ALL.iter().enumerate() {
            let option = u32::try_from(index).unwrap_or(u32::MAX);
            assert_eq!(column_for_option(option), Some(*column));
        }
        let past_end = u32::try_from(ColumnId::COUNT).unwrap_or(u32::MAX);
        assert_eq!(column_for_option(past_end), None);
        assert_eq!(column_for_option(u32::MAX), None);
    }

    #[test]
    fn the_list_fits_inside_the_menu_height() {
        for row_height in [CHECKLIST_ROW_HEIGHT, CHECKLIST_ROW_HEIGHT_TOUCH] {
            let rows = visible_rows(row_height);
            assert!(rows >= 1);
            assert!(rows < ColumnId::COUNT);
            #[allow(clippy::cast_precision_loss, reason = "a handful of rows")]
            let list_height = rows as f32 * row_height;
            assert!(list_height < MENU_MAX_HEIGHT);
        }
    }

    #[test]
    fn a_degenerate_row_height_still_shows_a_row() {
        assert_eq!(visible_rows(f32::INFINITY), 1);
        assert_eq!(visible_rows(f32::NAN), 1);
        assert_eq!(visible_rows(10_000.0), 1);
    }

    #[test]
    fn the_menu_builds_for_every_context() {
        let mut columns = Columns::default();
        for ctx in contexts() {
            let _full = view(ctx, &columns);
        }
        for column in ColumnId::ALL.iter().skip(1) {
            assert!(columns.set_visible(*column, false));
        }
        assert_eq!(columns.visible_count(), 1);
        for ctx in contexts() {
            let _hinted = view(ctx, &columns);
        }
    }
}
