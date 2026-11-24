//! Rail text fields: exact colour and the three numeric ranges.

use iced::widget::Column;
use iced::{Element, Length};

use crate::app::context::ViewContext;
use crate::app::i18n::Text;
use crate::app::ids;
use crate::app::layout::{RANGE_GAP, inline_row};
use crate::app::message::{FilterMessage, Message};
use crate::app::panes::controls::{field_error, small_label, text_field, with_tooltip};
use crate::app::state::{FieldBuffer, TextFields};
use crate::app::view_model::{Bound, RangeField};

/// Space between a label, its field and its error.
const LABEL_GAP: f32 = RANGE_GAP / 2.0;

/// Exact-colour field with its label, tooltip and inline error.
#[must_use]
pub(super) fn rgb_field<'a>(ctx: ViewContext, buffer: &'a FieldBuffer) -> Element<'a, Message> {
    let field = text_field(
        ctx,
        ids::RGB_FIELD,
        ctx.t(Text::RgbPlaceholder),
        &buffer.text,
        buffer.error.is_some(),
        |text| Message::Filter(FilterMessage::Rgb(text)),
    );
    let mut column = Column::new()
        .spacing(LABEL_GAP)
        .width(Length::Fill)
        .push(small_label(ctx, ctx.t(Text::RgbLabel)))
        .push(with_tooltip(ctx, field, ctx.t(Text::RgbTooltip)));
    if let Some(error) = buffer.error {
        column = column.push(field_error(ctx, ctx.t(error)));
    }
    column.into()
}

/// Minimum and maximum fields of one range under its label, with the inline error below.
#[must_use]
pub(super) fn range_row<'a>(
    ctx: ViewContext,
    field: RangeField,
    fields: &'a TextFields,
) -> Element<'a, Message> {
    let mut cells: Vec<Element<'a, Message>> = Vec::with_capacity(2);
    let mut errors: Vec<Element<'a, Message>> = Vec::with_capacity(2);
    for bound in [Bound::Min, Bound::Max] {
        let buffer = fields.range(field, bound);
        cells.push(text_field(
            ctx,
            ids::range_field(field, bound),
            ctx.t(bound.placeholder()),
            &buffer.text,
            buffer.error.is_some(),
            move |text| Message::Filter(FilterMessage::Range { field, bound, text }),
        ));
        if let Some(error) = buffer.error {
            errors.push(field_error(ctx, ctx.t(error)));
        }
    }
    let mut column = Column::new()
        .spacing(LABEL_GAP)
        .width(Length::Fill)
        .push(small_label(ctx, ctx.t(field.title())))
        .push(
            inline_row(ctx.direction, cells)
                .spacing(RANGE_GAP)
                .width(Length::Fill),
        );
    for error in errors {
        column = column.push(error);
    }
    column.into()
}
