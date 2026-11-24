//! Breadcrumb, field group and field row builders of the detail pane.

use iced::alignment::Vertical;
use iced::font::Weight;
use iced::widget::text::Wrapping;
use iced::widget::{Column, container};
use iced::{Element, Length, Padding};

use crate::app::context::ViewContext;
use crate::app::layout::{
    DETAIL_FIELD_MIN_HEIGHT, DETAIL_GROUP_GAP, DETAIL_LABEL_WIDTH, DIALOG_GAP, MENU_PADDING,
    inline_row,
};
use crate::app::message::Message;
use crate::app::panes::overlays::kit::{mono, prose, sans, wrapped};
use crate::app::theme::{
    Icon, LINE_HEIGHT_PROSE, TEXT_BODY, TEXT_CAPTION, TEXT_META, TEXT_SMALL,
};
use crate::app::view_model::{DetailField, DetailGroup, tone_color};

/// Breadcrumb levels worth showing: a missing level is an empty label and is skipped, so the
/// separators never double up.
#[must_use]
pub(super) fn breadcrumb_levels(levels: &[String]) -> Vec<&str> {
    levels
        .iter()
        .map(String::as_str)
        .filter(|level| !level.is_empty())
        .collect()
}

/// Vertical padding that lifts a one-line value row to `DETAIL_FIELD_MIN_HEIGHT`.
#[must_use]
pub(super) fn field_padding() -> f32 {
    ((DETAIL_FIELD_MIN_HEIGHT - TEXT_BODY * LINE_HEIGHT_PROSE) / 2.0).max(0.0)
}

/// Hierarchy from continent to province: labels in `ink_muted` with direction-aware `ink_faint`
/// separators, wrapping onto further lines in a narrow pane.
pub(super) fn breadcrumb<'a>(ctx: ViewContext, levels: &'a [String]) -> Element<'a, Message> {
    let palette = ctx.palette;
    let separator = Icon::Breadcrumb;
    let mut children: Vec<Element<'a, Message>> = Vec::new();
    for (index, level) in breadcrumb_levels(levels).into_iter().enumerate() {
        if index > 0 {
            children.push(
                iced::widget::text(separator.glyph(ctx.direction).to_string())
                    .size(TEXT_SMALL)
                    .font(separator.font())
                    .color(palette.ink_faint)
                    .into(),
            );
        }
        children.push(sans(ctx, level, TEXT_SMALL, Weight::Normal, palette.ink_muted).into());
    }
    wrapped(ctx.direction, children, MENU_PADDING / 2.0, 2.0)
}

/// One titled group: caption, label and value rows, and the optional note.
pub(super) fn group_view<'a>(ctx: ViewContext, group: &'a DetailGroup) -> Element<'a, Message> {
    let palette = ctx.palette;
    let mut column = Column::new()
        .push(
            mono(ctx, ctx.t(group.title), TEXT_CAPTION, true, palette.ink_faint)
                .width(Length::Fill),
        )
        .width(Length::Fill);
    for field in &group.fields {
        column = column.push(field_row(ctx, field));
    }
    if let Some(note) = group.note {
        column = column.push(prose(ctx, ctx.t(note), TEXT_SMALL, palette.ink_muted));
    }
    // The column spacing already puts `MENU_PADDING` above the group.
    container(column)
        .padding(Padding::ZERO.top((DETAIL_GROUP_GAP - MENU_PADDING).max(0.0)))
        .width(Length::Fill)
        .into()
}

/// Label column and wrapping value; technical values use the mono face.
fn field_row<'a>(ctx: ViewContext, field: &'a DetailField) -> Element<'a, Message> {
    let palette = ctx.palette;
    let label = sans(ctx, ctx.t(field.label), TEXT_SMALL, Weight::Normal, palette.ink_muted)
        .width(Length::Fixed(DETAIL_LABEL_WIDTH))
        .wrapping(Wrapping::WordOrGlyph);
    let color = tone_color(palette, field.tone);
    let value = if field.mono {
        mono(ctx, field.value.as_str(), TEXT_META, false, color)
            .width(Length::Fill)
            .wrapping(Wrapping::WordOrGlyph)
    } else {
        prose(ctx, field.value.as_str(), TEXT_BODY, color)
    };
    inline_row(ctx.direction, vec![label.into(), value.into()])
        .spacing(DIALOG_GAP)
        .padding(Padding::ZERO.vertical(field_padding()))
        .width(Length::Fill)
        .align_y(Vertical::Top)
        .into()
}

