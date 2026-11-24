//! Shared control builders used by every pane, so buttons, fields, badges and menus look and
//! behave the same everywhere. Buttons and toggles come wrapped in a `FocusRing`; callers add
//! an identifier with `.id(..)` when focus must return to the control.

mod buttons;
mod fields;
mod menu;
mod text;
#[cfg(test)]
mod tests;

pub(crate) use buttons::{button_frame, button_label, icon_button, text_button};
pub(crate) use fields::{field_error, text_field, text_field_sized, toggle_row};
pub(crate) use menu::menu_frame;
pub(crate) use text::{
    badge, caption, counted, mono_font, sans_font, small_label, with_tooltip, with_tooltip_owned,
};
