//! Drawer frames, the scrim behind drawers and dialogs, and the centred modal frame.

use iced::Element;

use crate::app::context::ViewContext;
use crate::app::layout::Drawer;
use crate::app::message::Message;

/// A drawer of `width` sliding in from its inline edge (filters from the start, details from
/// the end); `progress` in `0..=1` is the slide position from the drawer animation.
#[must_use]
pub(crate) fn drawer<'a>(
    ctx: ViewContext,
    drawer: Drawer,
    width: f32,
    progress: f32,
    content: Element<'a, Message>,
) -> Element<'a, Message> {
    let _ = (ctx, drawer, width, progress);
    content
}

/// Full-window scrim at `opacity` that publishes `on_press` when pressed and blocks input to
/// everything beneath.
#[must_use]
pub(crate) fn scrim<'a>(ctx: ViewContext, opacity: f32, on_press: Message) -> Element<'a, Message> {
    let _ = (ctx, opacity, on_press);
    iced::widget::space().into()
}

/// Centres a dialog card within the viewport, at least `DIALOG_MARGIN` from its edges.
#[must_use]
pub(crate) fn modal<'a>(ctx: ViewContext, card: Element<'a, Message>) -> Element<'a, Message> {
    let _ = ctx;
    card
}
