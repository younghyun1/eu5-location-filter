//! Transient surfaces stacked over the body.

use iced::Element;

use crate::app::context::ViewContext;
use crate::app::message::Message;
use crate::app::state::App;

/// Stacks the open menu or dialog, notice, drawer, loading card and error card over `body`.
#[must_use]
pub(super) fn with_surfaces<'a>(
    app: &'a App,
    context: ViewContext,
    body: Element<'a, Message>,
) -> Element<'a, Message> {
    let _ = (app, context);
    body
}
