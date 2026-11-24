//! Load-error card over the error veil.

use iced::Element;

use crate::app::context::ViewContext;
use crate::app::message::Message;
use crate::app::state::LoadFailure;

/// Title, family message for the error variant, technical detail, hint, and on desktop the
/// installation directory field with Retry and Rebuild data.
#[must_use]
pub(crate) fn error_card<'a>(
    ctx: ViewContext,
    failure: &'a LoadFailure,
    install_path: &'a str,
) -> Element<'a, Message> {
    let _ = (ctx, failure, install_path);
    iced::widget::space().into()
}
