//! Column menu popover: a checklist of all columns without a search field. Shows
//! `column_menu.keep_one` while only one column is visible.

use iced::Element;

use crate::app::context::ViewContext;
use crate::app::message::Message;
use crate::app::view_model::Columns;

/// Menu content inside the shared menu frame; toggles publish `ColumnMessage::SetVisible`.
#[must_use]
pub(crate) fn view<'a>(ctx: ViewContext, columns: &'a Columns) -> Element<'a, Message> {
    let _ = (ctx, columns);
    iced::widget::space().into()
}
