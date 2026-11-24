//! Text and icon buttons wrapped in a focus ring, with the full label as a tooltip.

use iced::font::Weight;
use iced::widget::text::Wrapping;
use iced::widget::{button, container, text};
use iced::{Element, Length, Padding, Size};

use super::text::{sans_font, with_tooltip};
use crate::app::context::ViewContext;
use crate::app::layout::{
    BUTTON_HEIGHT, BUTTON_PADDING_INLINE, ICON_BUTTON_SIZE, TOUCH_TARGET,
};
use crate::app::message::Message;
use crate::app::theme::{self, ButtonKind, Icon, TEXT_BODY};
use crate::app::widget::{FocusRing, focus_ring};

/// Height of text buttons: `BUTTON_HEIGHT`, or `TOUCH_TARGET` in the narrow layout.
fn button_height(ctx: ViewContext) -> f32 {
    if ctx.is_narrow() {
        TOUCH_TARGET
    } else {
        BUTTON_HEIGHT
    }
}

/// Button label in Sans Medium `TEXT_BODY` that never wraps; the button clips what does not fit.
#[must_use]
pub(crate) fn button_label<'a>(ctx: ViewContext, label: &'a str) -> Element<'a, Message> {
    text(label)
        .size(TEXT_BODY)
        .font(sans_font(ctx, Weight::Medium))
        .wrapping(Wrapping::None)
        .into()
}

/// Fixed-size text button; the label clips and the full text shows as a tooltip. `None` for
/// `on_press` draws it disabled.
#[must_use]
pub(crate) fn text_button<'a>(
    ctx: ViewContext,
    label: &'a str,
    kind: ButtonKind,
    width: f32,
    on_press: Option<Message>,
) -> FocusRing<'a, Message> {
    button_frame(
        ctx,
        button_label(ctx, label),
        label,
        kind,
        Length::Fixed(width),
        on_press,
    )
}

/// Button of the shared height around any `content`, centred in the button. Used directly where
/// the width is not fixed (equal cells of the narrow header) or the content is more than a label.
#[must_use]
pub(crate) fn button_frame<'a>(
    ctx: ViewContext,
    content: Element<'a, Message>,
    tip: &'a str,
    kind: ButtonKind,
    width: Length,
    on_press: Option<Message>,
) -> FocusRing<'a, Message> {
    frame(
        ctx,
        content,
        tip,
        kind,
        Size::new(width, Length::Fixed(button_height(ctx))),
        on_press,
    )
}

/// Square icon button with a tooltip.
#[must_use]
pub(crate) fn icon_button<'a>(
    ctx: ViewContext,
    icon: Icon,
    tooltip: &'a str,
    on_press: Option<Message>,
) -> FocusRing<'a, Message> {
    let side = if ctx.is_narrow() {
        TOUCH_TARGET
    } else {
        ICON_BUTTON_SIZE
    };
    let glyph = text(icon.glyph(ctx.direction).to_string())
        .size(TEXT_BODY)
        .font(icon.font());
    frame(
        ctx,
        glyph.into(),
        tooltip,
        ButtonKind::Ghost,
        Size::new(Length::Fixed(side), Length::Fixed(side)),
        on_press,
    )
}

/// The button, its tooltip and its focus ring. The press message is also the ring's activation
/// message, so Enter and Space on the focused ring do what a click does.
fn frame<'a>(
    ctx: ViewContext,
    content: Element<'a, Message>,
    tip: &'a str,
    kind: ButtonKind,
    size: Size<Length>,
    on_press: Option<Message>,
) -> FocusRing<'a, Message> {
    let palette = ctx.palette;
    let padding = if matches!(kind, ButtonKind::Ghost) {
        Padding::ZERO
    } else {
        Padding::from([0.0, BUTTON_PADDING_INLINE])
    };
    let body = button(container(content).center(Length::Fill))
        .width(size.width)
        .height(size.height)
        .padding(padding)
        .clip(true)
        .on_press_maybe(on_press.clone())
        .style(move |_theme, status| theme::button(palette, kind, status));
    focus_ring(with_tooltip(ctx, body.into(), tip))
        .palette(palette)
        .on_activate_maybe(on_press)
}
