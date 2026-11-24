//! Loading card over the loading veil.

use iced::Element;

use crate::app::context::ViewContext;
use crate::app::message::Message;
use crate::app::state::LoadProgress;

/// Stage text, progress bar and `format.progress`; the bar reflects reported progress only.
#[must_use]
pub(crate) fn loading_card<'a>(ctx: ViewContext, progress: LoadProgress) -> Element<'a, Message> {
    let _ = (ctx, progress);
    iced::widget::space().into()
}
