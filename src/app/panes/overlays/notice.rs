//! Notice bar under the header.

use iced::Element;

use crate::app::context::ViewContext;
use crate::app::message::Message;
use crate::app::state::Notice;

/// Message, `danger` inline-start rule and Dismiss.
#[must_use]
pub(crate) fn notice_bar<'a>(ctx: ViewContext, notice: Notice) -> Element<'a, Message> {
    let _ = (ctx, notice);
    iced::widget::space().into()
}
