//! Results toolbar: legend, Columns button, and Show details while the detail pane is hidden.

use iced::Element;

use crate::app::context::ViewContext;
use crate::app::message::Message;

/// Toolbar above the grid; the narrow layout drops the legend.
#[must_use]
pub(crate) fn view<'a>(
    ctx: ViewContext,
    detail_hidden: bool,
    column_menu_open: bool,
) -> Element<'a, Message> {
    let _ = (ctx, detail_hidden, column_menu_open);
    iced::widget::space().into()
}
