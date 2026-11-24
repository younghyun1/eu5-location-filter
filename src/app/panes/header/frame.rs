//! Ground and bottom hairline shared by both header layouts.

use iced::widget::{Column, container, rule};
use iced::{Element, Length};

use crate::app::context::ViewContext;
use crate::app::layout::BORDER_WIDTH;
use crate::app::message::Message;
use crate::app::theme;

/// Puts `content` on the header ground with a `line` hairline below it. `height` is the total
/// header height, so the content area is one hairline shorter.
#[must_use]
pub(super) fn framed<'a>(
    ctx: ViewContext,
    content: Element<'a, Message>,
    height: f32,
) -> Element<'a, Message> {
    let palette = ctx.palette;
    let bar = container(content)
        .width(Length::Fill)
        .height(height - BORDER_WIDTH)
        .style(move |_theme| theme::header(palette));
    let line = rule::horizontal(BORDER_WIDTH).style(move |_theme| theme::rule(palette, false));
    Column::new()
        .push(bar)
        .push(line)
        .width(Length::Fill)
        .into()
}
