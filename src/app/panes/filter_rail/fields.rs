//! Rail text fields: exact colour and the three numeric ranges.

use iced::Element;

use crate::app::context::ViewContext;
use crate::app::message::Message;
use crate::app::state::{FieldBuffer, TextFields};
use crate::app::view_model::RangeField;

/// Exact-colour field with its label, tooltip and inline error.
#[must_use]
pub(super) fn rgb_field<'a>(ctx: ViewContext, buffer: &'a FieldBuffer) -> Element<'a, Message> {
    let _ = (ctx, buffer);
    iced::widget::space().into()
}

/// Minimum and maximum fields of one range under its label, with the inline error below.
#[must_use]
pub(super) fn range_row<'a>(
    ctx: ViewContext,
    field: RangeField,
    fields: &'a TextFields,
) -> Element<'a, Message> {
    let _ = (ctx, field, fields);
    iced::widget::space().into()
}
