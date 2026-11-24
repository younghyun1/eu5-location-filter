//! Text fields, toggles and inline field errors.

use iced::alignment::Vertical;
use iced::font::Weight;
use iced::widget::text::LineHeight;
use iced::widget::{Id, checkbox, container, mouse_area, text, text_input};
use iced::{Element, Length, Padding, mouse};

use super::text::sans_font;
use crate::app::context::ViewContext;
use crate::app::layout::anchored_text;
use crate::app::layout::{
    CHECK_LABEL_GAP, CHECKBOX_SIZE, INPUT_HEIGHT, INPUT_PADDING_INLINE, TOUCH_TARGET, inline_row,
};
use crate::app::message::Message;
use crate::app::theme::{self, LINE_HEIGHT, TEXT_BODY, TEXT_INPUT_NARROW, TEXT_SMALL};
use crate::app::widget::focus_ring;

/// Text field of the shared height; `invalid` draws a `danger` border. Aligns its text to the
/// inline start.
#[must_use]
pub(crate) fn text_field<'a>(
    ctx: ViewContext,
    id: Id,
    placeholder: &'a str,
    value: &'a str,
    invalid: bool,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    let height = if ctx.is_narrow() {
        TOUCH_TARGET
    } else {
        INPUT_HEIGHT
    };
    text_field_sized(ctx, id, placeholder, value, invalid, height, on_input)
}

/// `text_field` with an explicit outer height, for the taller header search field. The text is
/// centred by padding computed from the height and the relative line height, so the field is
/// exactly `height` high whatever the label font.
#[must_use]
pub(crate) fn text_field_sized<'a>(
    ctx: ViewContext,
    id: Id,
    placeholder: &'a str,
    value: &'a str,
    invalid: bool,
    height: f32,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    let palette = ctx.palette;
    let size = if ctx.is_narrow() {
        TEXT_INPUT_NARROW
    } else {
        TEXT_BODY
    };
    let padding_block = ((height - size * LINE_HEIGHT) / 2.0).max(0.0);
    text_input(placeholder, value)
        .id(id)
        .on_input(on_input)
        .size(size)
        .line_height(LineHeight::Relative(LINE_HEIGHT))
        .padding(Padding::from([padding_block, INPUT_PADDING_INLINE]))
        .font(sans_font(ctx, Weight::Normal))
        .align_x(ctx.direction.start())
        .width(Length::Fill)
        .style(move |_theme, status| theme::text_input(palette, invalid, status))
        .into()
}

/// Checkbox row for rail toggles, checkbox at the inline start. The label is part of the hit
/// area, and the focus ring's activation toggles it from the keyboard.
#[must_use]
pub(crate) fn toggle_row<'a>(
    ctx: ViewContext,
    label: &'a str,
    checked: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
) -> Element<'a, Message> {
    let palette = ctx.palette;
    let height = if ctx.is_narrow() {
        TOUCH_TARGET
    } else {
        INPUT_HEIGHT
    };
    // The checkbox itself publishes through `on_toggle`; a press on the label has no checkbox
    // under it, so the same outcome is precomputed for the mouse area and the ring.
    let on_label = on_toggle(!checked);
    let boxed = checkbox(checked)
        .size(CHECKBOX_SIZE)
        .on_toggle(on_toggle)
        .style(move |_theme, status| theme::checkbox(palette, status));
    let caption: Element<'a, Message> = anchored_text(
        text(label)
            .size(TEXT_BODY)
            .font(sans_font(ctx, Weight::Normal))
            .color(palette.ink),
        ctx.direction.start(),
    );
    let row = inline_row(ctx.direction, vec![boxed.into(), caption])
        .spacing(CHECK_LABEL_GAP)
        .align_y(Vertical::Center)
        .width(Length::Fill);
    let hit = mouse_area(container(row).width(Length::Fill).center_y(height))
        .on_press(on_label.clone())
        .interaction(mouse::Interaction::Pointer);
    focus_ring(hit)
        .palette(palette)
        .on_activate(on_label)
        .into()
}

/// Inline error message under a field, Sans `TEXT_SMALL` `danger`.
#[must_use]
pub(crate) fn field_error<'a>(ctx: ViewContext, message: &'a str) -> Element<'a, Message> {
    anchored_text(
        text(message)
            .size(TEXT_SMALL)
            .font(sans_font(ctx, Weight::Normal))
            .color(ctx.palette.danger),
        ctx.direction.start(),
    )
}
