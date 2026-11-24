//! Locale, palette and motion changes from either the desktop menus or the host, and
//! animation frames.

use iced::Task;
use iced::time::Instant;

use super::filters::{rebuild_detail, rebuild_options};
use super::packs::request_packs;
use crate::app::i18n::Locale;
use crate::app::message::{Message, SettingsMessage};
use crate::app::state::{App, drawer_animation};
use crate::app::theme::{Palette, PaletteFade};

/// Desktop Language and Appearance choices.
pub(super) fn settings(app: &mut App, message: SettingsMessage) -> Task<Message> {
    match message {
        SettingsMessage::Locale(locale) => set_locale(app, locale),
        SettingsMessage::Mode(mode) => {
            app.appearance.mode = mode;
            apply_desktop_palette(app);
            Task::none()
        }
        SettingsMessage::Accent(accent) => {
            app.appearance.accent = accent;
            apply_desktop_palette(app);
            Task::none()
        }
        SettingsMessage::Motion(enabled) => {
            app.appearance.motion_enabled = enabled;
            apply_reduced_motion(app, !enabled);
            Task::none()
        }
    }
}

/// Built-in palette of the chosen mode with the chosen accent.
fn apply_desktop_palette(app: &mut App) {
    let palette = Palette::builtin(app.appearance.mode).with_accent(app.appearance.accent);
    set_palette(app, palette);
}

/// Applies a locale: direction, catalog text, option labels, and the game language and font
/// pack it needs (each requested at most once, never refetching a permanent failure).
pub(super) fn set_locale(app: &mut App, locale: Locale) -> Task<Message> {
    if locale != app.locale {
        app.locale = locale;
        app.direction = locale.direction();
        rebuild_options(app);
        rebuild_detail(app);
    }
    request_packs(app)
}

/// Starts a fade to a new palette, or applies it at once with reduced motion. A repeat of the
/// current target, which the host sends with every settings message, changes nothing.
pub(super) fn set_palette(app: &mut App, palette: Palette) {
    if palette == app.appearance.fade.target() {
        return;
    }
    // The subscription reads `appearance.now` to decide whether frames are needed, so the
    // clock moves to the start of the fade.
    let now = Instant::now();
    app.appearance.now = now;
    app.appearance
        .fade
        .retarget(palette, now, app.appearance.reduced_motion);
    sample_palette(app, now);
}

/// Sets the effective reduced-motion flag. Animation durations are fixed when an animation is
/// built, so the drawer animation is rebuilt at its current target and a running palette fade
/// is settled.
pub(super) fn apply_reduced_motion(app: &mut App, reduced: bool) {
    if app.appearance.reduced_motion == reduced {
        return;
    }
    app.appearance.reduced_motion = reduced;
    let open = app.overlay.drawer_motion.value();
    app.overlay.drawer_motion = drawer_animation(open, reduced);
    if !open {
        app.panes.drawer = None;
    }
    if reduced {
        let target = app.appearance.fade.target();
        app.appearance.fade = PaletteFade::settled(target);
        let now = app.appearance.now;
        sample_palette(app, now);
    }
}

/// Samples the fade and the drawer animation at `now` and rebuilds the iced theme when the
/// palette changed.
pub(super) fn frame(app: &mut App, now: Instant) -> Task<Message> {
    app.appearance.now = now;
    sample_palette(app, now);
    let motion = &app.overlay.drawer_motion;
    if app.panes.drawer.is_some() && !motion.value() && !motion.is_animating(now) {
        app.panes.drawer = None;
    }
    Task::none()
}

/// Draws the fade's palette at `now`; the iced theme is rebuilt only when the palette changed,
/// because building one generates a full extended palette.
fn sample_palette(app: &mut App, now: Instant) {
    let palette = app.appearance.fade.sample(now);
    if palette != app.appearance.palette {
        app.appearance.palette = palette;
        app.appearance.theme = palette.iced_theme();
    }
}
