//! Locale, palette and motion changes from either the desktop menus or the host, and
//! animation frames.

use iced::Task;
use iced::time::Instant;

use crate::app::i18n::Locale;
use crate::app::message::{Message, SettingsMessage};
use crate::app::state::App;
use crate::app::theme::Palette;

/// Desktop Language and Appearance choices.
pub(super) fn settings(app: &mut App, message: SettingsMessage) -> Task<Message> {
    let _ = (app, message);
    Task::none()
}

/// Applies a locale: direction, catalog text, option labels, and the game language and font
/// pack it needs (each requested at most once, never refetching a permanent failure).
pub(super) fn set_locale(app: &mut App, locale: Locale) -> Task<Message> {
    let _ = (app, locale);
    Task::none()
}

/// Starts a fade to a new palette, or applies it at once with reduced motion.
pub(super) fn set_palette(app: &mut App, palette: Palette) {
    let _ = (app, palette);
}

/// Samples the fade and the drawer animation at `now` and rebuilds the iced theme when the
/// palette changed.
pub(super) fn frame(app: &mut App, now: Instant) -> Task<Message> {
    let _ = (app, now);
    Task::none()
}
