//! Results toolbar: legend, Columns button, and Show details while the detail pane is hidden.

use iced::alignment::Vertical;
use iced::widget::text::Wrapping;
use iced::widget::{Column, container, rule, space, text};
use iced::{Background, Color, Element, Length, Padding};

use crate::app::context::ViewContext;
use crate::app::i18n::Text;
use crate::app::ids;
use crate::app::layout::{
    BORDER_WIDTH, BUTTON_WIDTH, GRID_TOOLBAR_HEIGHT, HEADER_GAP, HEADER_PADDING_INLINE,
    LEGEND_MARK_SIZE, NARROW_PADDING_INLINE, NARROW_TOOLBAR_HEIGHT, RADIUS, RANGE_GAP, inline_row,
};
use crate::app::message::{Message, OverlayMessage, PaneMessage};
use crate::app::panes::controls::{mono_font, text_button};
use crate::app::state::Surface;
use crate::app::theme::{self, ButtonKind, TEXT_CAPTION};

/// Toolbar above the grid; the narrow layout drops the legend. `detail_hidden` adds Show
/// details in the wide layout, and `column_menu_open` draws the Columns button as the open
/// menu's opener.
#[must_use]
pub(crate) fn view<'a>(
    ctx: ViewContext,
    detail_hidden: bool,
    column_menu_open: bool,
) -> Element<'a, Message> {
    let palette = ctx.palette;
    let narrow = ctx.is_narrow();
    let (height, inset) = if narrow {
        (NARROW_TOOLBAR_HEIGHT, NARROW_PADDING_INLINE)
    } else {
        (GRID_TOOLBAR_HEIGHT, HEADER_PADDING_INLINE)
    };

    // The legend takes the space the buttons leave and clips when there is not enough, so the
    // buttons never move.
    let leading: Element<'a, Message> = if narrow {
        space::horizontal().into()
    } else {
        container(legend(ctx))
            .width(Length::Fill)
            .align_x(ctx.direction.start())
            .center_y(Length::Fill)
            .clip(true)
            .into()
    };
    let columns_kind = if column_menu_open {
        ButtonKind::Primary
    } else {
        ButtonKind::Secondary
    };
    let mut items: Vec<Element<'a, Message>> = vec![
        leading,
        text_button(
            ctx,
            ctx.t(Text::GridColumns),
            columns_kind,
            BUTTON_WIDTH,
            Some(Message::Overlay(OverlayMessage::Toggle(Surface::ColumnMenu))),
        )
        .id(ids::COLUMNS_BUTTON)
        .into(),
    ];
    if detail_hidden && !narrow {
        items.push(
            text_button(
                ctx,
                ctx.t(Text::GridShowDetails),
                ButtonKind::Secondary,
                BUTTON_WIDTH,
                Some(Message::Pane(PaneMessage::SetDetailHidden(false))),
            )
            .id(ids::SHOW_DETAILS_BUTTON)
            .into(),
        );
    }
    let row = inline_row(ctx.direction, items)
        .spacing(RANGE_GAP)
        .align_y(Vertical::Center)
        .padding(Padding::from([0.0, inset]))
        .width(Length::Fill)
        .height(Length::Fill);
    let bar = container(row)
        .width(Length::Fill)
        .height(height - BORDER_WIDTH)
        .style(move |_theme| theme::toolbar(palette));
    let line = rule::horizontal(BORDER_WIDTH).style(move |_theme| theme::rule(palette, false));
    Column::new()
        .push(bar)
        .push(line)
        .width(Length::Fill)
        .into()
}

/// Legend: a colour square and caption for food-producing goods and for gold or silver.
fn legend<'a>(ctx: ViewContext) -> Element<'a, Message> {
    let palette = ctx.palette;
    let entries = [
        (palette.food(), Text::GridLegendFood),
        (palette.precious(), Text::GridLegendPrecious),
    ];
    let mut items: Vec<Element<'a, Message>> = Vec::with_capacity(entries.len());
    for (color, label) in entries {
        items.push(legend_entry(ctx, color, ctx.t(label)));
    }
    inline_row(ctx.direction, items)
        .spacing(HEADER_GAP)
        .align_y(Vertical::Center)
        .into()
}

/// One legend entry in inline order: mark, then caption.
fn legend_entry<'a>(ctx: ViewContext, color: Color, label: &'a str) -> Element<'a, Message> {
    let mark = container(space())
        .width(LEGEND_MARK_SIZE)
        .height(LEGEND_MARK_SIZE)
        .style(move |_theme| container::Style {
            background: Some(Background::Color(color)),
            border: iced::Border {
                radius: RADIUS.into(),
                ..iced::Border::default()
            },
            ..container::Style::default()
        });
    let caption = text(label)
        .size(TEXT_CAPTION)
        .font(mono_font(ctx, true))
        .color(ctx.palette.ink_muted)
        .wrapping(Wrapping::None);
    inline_row(ctx.direction, vec![mark.into(), caption.into()])
        .spacing(RANGE_GAP)
        .align_y(Vertical::Center)
        .into()
}

#[cfg(test)]
mod tests {
    use iced::Element;

    use super::view;
    use crate::app::context::{Platform, ViewContext};
    use crate::app::i18n::Locale;
    use crate::app::layout::{Direction, LayoutMode};
    use crate::app::message::Message;
    use crate::app::theme::PALETTE_DARK;

    #[test]
    fn toolbar_builds_for_every_state() {
        for direction in [Direction::Ltr, Direction::Rtl] {
            for mode in [LayoutMode::Wide, LayoutMode::Narrow] {
                for (hidden, open) in [(false, false), (true, false), (true, true)] {
                    let ctx = ViewContext {
                        palette: PALETTE_DARK,
                        locale: Locale::EnUs,
                        direction,
                        mode,
                        platform: Platform::Desktop,
                    };
                    let _: Element<'_, Message> = view(ctx, hidden, open);
                }
            }
        }
    }
}
