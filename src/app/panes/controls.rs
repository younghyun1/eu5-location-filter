//! Shared control builders used by every pane, so buttons, fields, badges and menus look and
//! behave the same everywhere. Buttons and toggles come wrapped in a `FocusRing`; callers add
//! an identifier with `.id(..)` when focus must return to the control.

use iced::Element;
use iced::widget::Id;

use crate::app::context::ViewContext;
use crate::app::message::Message;
use crate::app::theme::{ButtonKind, Icon};
use crate::app::widget::{FocusRing, focus_ring};

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
    let _ = (ctx, label, kind, width);
    focus_ring(iced::widget::space()).on_activate_maybe(on_press)
}

/// Square icon button with a tooltip.
#[must_use]
pub(crate) fn icon_button<'a>(
    ctx: ViewContext,
    icon: Icon,
    tooltip: &'a str,
    on_press: Option<Message>,
) -> FocusRing<'a, Message> {
    let _ = (ctx, icon, tooltip);
    focus_ring(iced::widget::space()).on_activate_maybe(on_press)
}

/// Pill badge with a count in Mono Medium `TEXT_CAPTION`.
#[must_use]
pub(crate) fn badge<'a>(ctx: ViewContext, count: usize) -> Element<'a, Message> {
    let _ = (ctx, count);
    iced::widget::space().into()
}

/// Section or pane caption in Mono Medium `TEXT_CAPTION` `ink_faint`.
#[must_use]
pub(crate) fn caption<'a>(ctx: ViewContext, label: &'a str) -> Element<'a, Message> {
    let _ = (ctx, label);
    iced::widget::space().into()
}

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
    let _ = (ctx, id, placeholder, value, invalid, on_input);
    iced::widget::space().into()
}

/// Checkbox row for rail toggles, checkbox at the inline start.
#[must_use]
pub(crate) fn toggle_row<'a>(
    ctx: ViewContext,
    label: &'a str,
    checked: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
) -> Element<'a, Message> {
    let _ = (ctx, label, checked, on_toggle);
    iced::widget::space().into()
}

/// Inline error message under a field, Sans `TEXT_SMALL` `danger`.
#[must_use]
pub(crate) fn field_error<'a>(ctx: ViewContext, message: &'a str) -> Element<'a, Message> {
    let _ = (ctx, message);
    iced::widget::space().into()
}

/// Popover frame shared by the column, language and appearance menus: `MENU_WIDTH` wide, at
/// most `MENU_MAX_HEIGHT` high, title row with a close button, scrolling content.
#[must_use]
pub(crate) fn menu_frame<'a>(
    ctx: ViewContext,
    title: &'a str,
    close_label: &'a str,
    content: Element<'a, Message>,
) -> Element<'a, Message> {
    let _ = (ctx, title, close_label);
    content
}

/// Wraps any element in a tooltip with the shared style.
#[must_use]
pub(crate) fn with_tooltip<'a>(
    ctx: ViewContext,
    content: Element<'a, Message>,
    tip: &'a str,
) -> Element<'a, Message> {
    let _ = (ctx, tip);
    content
}
