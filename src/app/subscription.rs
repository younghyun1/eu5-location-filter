//! Subscriptions: window size, application keyboard commands, Escape from captured events,
//! animation frames only while something animates, the first painted frame in the browser,
//! and the browser host inbox.
//!
//! iced identifies a subscription by its mapping function's type, so every mapping here is a
//! fn item or a non-capturing closure.

use iced::event::{self, Event};
use iced::keyboard::{self, Key, Modifiers, key};
use iced::time::Instant;
use iced::{Subscription, window};

use super::context::Platform;
use super::message::{KeyCommand, LoadMessage, Message};
use super::state::{App, Phase};

/// Every subscription the current state needs.
pub(crate) fn subscription(app: &App) -> Subscription<Message> {
    let mut subscriptions = Vec::with_capacity(6);
    subscriptions.push(event::listen_with(window_size));
    subscriptions.push(keyboard::listen().filter_map(key_command));
    subscriptions.push(event::listen_with(escape));
    if is_animating(app) {
        subscriptions.push(window::frames().map(Message::Frame));
    }
    if app.platform == Platform::Browser && matches!(app.phase, Phase::AwaitingPaint) {
        subscriptions.push(window::frames().map(painted));
    }
    #[cfg(all(feature = "web", target_family = "wasm"))]
    subscriptions.push(super::host::subscription());
    Subscription::batch(subscriptions)
}

/// Whether a palette fade or drawer slide still needs frames. The time of the latest frame is
/// used: an animation started after it reads as just begun, so frames start and then advance
/// `now` until it settles.
fn is_animating(app: &App) -> bool {
    let now = app.appearance.now;
    app.appearance.fade.is_animating(now) || app.overlay.drawer_motion.is_animating(now)
}

/// The first frame drawn in the browser; decoding the embedded bundles may start.
fn painted(_at: Instant) -> Message {
    Message::Load(LoadMessage::Painted)
}

/// Logical window size. `Opened` carries the size the window starts with, which matters in
/// the browser, where the canvas can be narrow before any resize happens; `Resized` follows.
fn window_size(event: Event, _status: event::Status, _window: window::Id) -> Option<Message> {
    match event {
        Event::Window(window::Event::Opened { size, .. } | window::Event::Resized(size)) => {
            Some(Message::WindowResized(size))
        }
        _ => None,
    }
}

/// Tab, Shift+Tab, Ctrl+F or Cmd+F (matched through `Key::to_latin` so non-Latin layouts work),
/// F6 and Shift+F6 from events no widget captured. Escape is read by `escape` instead, so it
/// is never reported twice.
fn key_command(event: keyboard::Event) -> Option<Message> {
    match event {
        keyboard::Event::KeyPressed {
            key,
            physical_key,
            modifiers,
            ..
        } => command_for(&key, physical_key, modifiers).map(Message::Key),
        keyboard::Event::KeyReleased { .. } | keyboard::Event::ModifiersChanged(_) => None,
    }
}

/// The command for one key press, if it is an application shortcut.
fn command_for(key: &Key, physical_key: key::Physical, modifiers: Modifiers) -> Option<KeyCommand> {
    match key.as_ref() {
        Key::Named(key::Named::Tab) if !modifiers.control() && !modifiers.alt() => {
            Some(if modifiers.shift() {
                KeyCommand::FocusPrevious
            } else {
                KeyCommand::FocusNext
            })
        }
        Key::Named(key::Named::F6) if !modifiers.control() && !modifiers.alt() => {
            Some(if modifiers.shift() {
                KeyCommand::PreviousRegion
            } else {
                KeyCommand::NextRegion
            })
        }
        Key::Character(_) if is_search_chord(modifiers) => match key.to_latin(physical_key) {
            Some(latin) if latin.eq_ignore_ascii_case(&'f') => Some(KeyCommand::FocusSearch),
            Some(_) | None => None,
        },
        Key::Named(_) | Key::Character(_) | Key::Unidentified => None,
    }
}

/// Ctrl, or Cmd on macOS, without Alt or Shift. iced picks the command modifier by compile
/// target, and the browser build never targets macOS, so there Cmd (reported as the logo key)
/// is accepted as well.
fn is_search_chord(modifiers: Modifiers) -> bool {
    let command = modifiers.command() || (cfg!(target_family = "wasm") && modifiers.logo());
    command && !modifiers.alt() && !modifiers.shift()
}

/// Escape from every event, captured or not, because a focused text input captures it to drop
/// its own focus and the same press must still close the topmost surface. Key repeat is
/// ignored so holding Escape closes one surface.
fn escape(event: Event, _status: event::Status, _window: window::Id) -> Option<Message> {
    match event {
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: Key::Named(key::Named::Escape),
            repeat: false,
            ..
        }) => Some(Message::Key(KeyCommand::Escape)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use iced::event::{self, Event};
    use iced::keyboard::key::{Code, Named, Physical};
    use iced::keyboard::{self, Key, Location, Modifiers};
    use iced::{Size, window};

    use super::{command_for, escape, key_command, window_size};
    use crate::app::message::{KeyCommand, Message};

    fn press(key: Key, code: Code, modifiers: Modifiers, repeat: bool) -> keyboard::Event {
        keyboard::Event::KeyPressed {
            modified_key: key.clone(),
            key,
            physical_key: Physical::Code(code),
            location: Location::Standard,
            modifiers,
            text: None,
            repeat,
        }
    }

    fn command(event: keyboard::Event) -> Option<KeyCommand> {
        match key_command(event) {
            Some(Message::Key(command)) => Some(command),
            Some(_) | None => None,
        }
    }

    #[test]
    fn tab_moves_focus_both_ways() {
        let tab = || Key::Named(Named::Tab);
        assert_eq!(
            command(press(tab(), Code::Tab, Modifiers::default(), false)),
            Some(KeyCommand::FocusNext)
        );
        assert_eq!(
            command(press(tab(), Code::Tab, Modifiers::SHIFT, true)),
            Some(KeyCommand::FocusPrevious)
        );
        assert_eq!(command(press(tab(), Code::Tab, Modifiers::ALT, false)), None);
    }

    #[test]
    fn f6_cycles_regions() {
        let f6 = || Key::Named(Named::F6);
        assert_eq!(
            command(press(f6(), Code::F6, Modifiers::default(), false)),
            Some(KeyCommand::NextRegion)
        );
        assert_eq!(
            command(press(f6(), Code::F6, Modifiers::SHIFT, false)),
            Some(KeyCommand::PreviousRegion)
        );
    }

    #[test]
    fn search_shortcut_works_on_latin_and_cyrillic_layouts() {
        let chord = if cfg!(target_os = "macos") {
            Modifiers::LOGO
        } else {
            Modifiers::CTRL
        };
        let latin = Key::Character("f".into());
        let cyrillic = Key::Character("а".into());
        assert_eq!(
            command_for(&latin, Physical::Code(Code::KeyF), chord),
            Some(KeyCommand::FocusSearch)
        );
        assert_eq!(
            command_for(&cyrillic, Physical::Code(Code::KeyF), chord),
            Some(KeyCommand::FocusSearch)
        );
        assert_eq!(
            command_for(&latin, Physical::Code(Code::KeyF), Modifiers::default()),
            None
        );
        assert_eq!(
            command_for(&latin, Physical::Code(Code::KeyF), chord | Modifiers::SHIFT),
            None
        );
        assert_eq!(
            command_for(&Key::Character("g".into()), Physical::Code(Code::KeyG), chord),
            None
        );
    }

    #[test]
    fn escape_is_left_to_the_event_listener() {
        let event = press(
            Key::Named(Named::Escape),
            Code::Escape,
            Modifiers::default(),
            false,
        );
        assert_eq!(command(event.clone()), None);
        let captured = escape(
            Event::Keyboard(event),
            event::Status::Captured,
            window::Id::unique(),
        );
        assert!(matches!(captured, Some(Message::Key(KeyCommand::Escape))));
    }

    #[test]
    fn escape_ignores_repeats_and_releases() {
        let repeated = press(
            Key::Named(Named::Escape),
            Code::Escape,
            Modifiers::default(),
            true,
        );
        let id = window::Id::unique();
        assert!(escape(Event::Keyboard(repeated), event::Status::Ignored, id).is_none());
        let released = keyboard::Event::KeyReleased {
            key: Key::Named(Named::Escape),
            modified_key: Key::Named(Named::Escape),
            physical_key: Physical::Code(Code::Escape),
            location: Location::Standard,
            modifiers: Modifiers::default(),
        };
        assert!(escape(Event::Keyboard(released), event::Status::Ignored, id).is_none());
    }

    #[test]
    fn window_size_follows_open_and_resize() {
        let id = window::Id::unique();
        let opened = Event::Window(window::Event::Opened {
            position: None,
            size: Size::new(360.0, 640.0),
        });
        assert!(matches!(
            window_size(opened, event::Status::Ignored, id),
            Some(Message::WindowResized(size)) if size == Size::new(360.0, 640.0)
        ));
        let resized = Event::Window(window::Event::Resized(Size::new(1200.0, 800.0)));
        assert!(matches!(
            window_size(resized, event::Status::Ignored, id),
            Some(Message::WindowResized(size)) if size == Size::new(1200.0, 800.0)
        ));
        assert!(window_size(Event::Window(window::Event::Focused), event::Status::Ignored, id).is_none());
    }
}
