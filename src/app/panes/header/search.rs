//! Global search slot: a text field on the desktop, an anchored empty slot in the browser.

use iced::widget::{container, space};
use iced::{Background, Border, Element, Length};

use crate::app::context::{Platform, ViewContext};
use crate::app::i18n::Text;
use crate::app::ids;
use crate::app::layout::{BORDER_WIDTH, RADIUS, SEARCH_HEIGHT, TOUCH_TARGET};
use crate::app::message::{FilterMessage, HostMessage, Message};
use crate::app::panes::controls::{text_field_sized, with_tooltip};
use crate::app::widget::anchor;

/// Search slot height: `SEARCH_HEIGHT` wide, `TOUCH_TARGET` narrow.
fn height(ctx: ViewContext) -> f32 {
    if ctx.is_narrow() {
        TOUCH_TARGET
    } else {
        SEARCH_HEIGHT
    }
}

/// The search control for the running build. The browser cannot receive input-method text on its
/// canvas, so there the application only reserves and reports the slot, and the host page places
/// a real text field over it.
#[must_use]
pub(super) fn search_slot<'a>(ctx: ViewContext, search: &'a str) -> Element<'a, Message> {
    match ctx.platform {
        Platform::Desktop => desktop_field(ctx, search),
        Platform::Browser => browser_slot(ctx),
    }
}

/// Desktop field with id `SEARCH`, filtering as the user types.
fn desktop_field<'a>(ctx: ViewContext, search: &'a str) -> Element<'a, Message> {
    let field = text_field_sized(
        ctx,
        ids::SEARCH,
        ctx.t(Text::HeaderSearchPlaceholder),
        search,
        false,
        height(ctx),
        |text| Message::Filter(FilterMessage::Search(text)),
    );
    with_tooltip(ctx, field, ctx.t(Text::HeaderSearchTooltip))
}

/// Empty field-shaped container whose window-space bounds go to the host.
fn browser_slot<'a>(ctx: ViewContext) -> Element<'a, Message> {
    let palette = ctx.palette;
    let slot = container(space())
        .width(Length::Fill)
        .height(height(ctx))
        .style(move |_theme| container::Style {
            background: Some(Background::Color(palette.surface)),
            border: Border {
                color: palette.line_strong,
                width: BORDER_WIDTH,
                radius: RADIUS.into(),
            },
            ..container::Style::default()
        });
    anchor(slot, |bounds| {
        Message::Host(HostMessage::SearchAnchor(bounds))
    })
    .into()
}
