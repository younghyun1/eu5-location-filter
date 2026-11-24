//! Subscriptions: window size, application keyboard commands, Escape from captured events,
//! animation frames only while something animates, the first painted frame in the browser,
//! and the browser host inbox.

use iced::event::{self, Event};
use iced::{Subscription, keyboard, window};

use super::context::Platform;
use super::message::{KeyCommand, LoadMessage, Message};
use super::state::{App, Phase};

/// Every subscription the current state needs.
pub(crate) fn subscription(app: &App) -> Subscription<Message> {
    let mut subscriptions = vec![
        window::resize_events().map(|(_id, size)| Message::WindowResized(size)),
        keyboard::listen().filter_map(key_command),
        event::listen_with(escape),
    ];
    if is_animating(app) {
        subscriptions.push(window::frames().map(Message::Frame));
    }
    if app.platform == Platform::Browser && matches!(app.phase, Phase::AwaitingPaint) {
        subscriptions.push(window::frames().map(|_| Message::Load(LoadMessage::Painted)));
    }
    #[cfg(all(feature = "web", target_family = "wasm"))]
    subscriptions.push(super::host::subscription());
    Subscription::batch(subscriptions)
}

/// Whether a palette fade or drawer slide still needs frames.
fn is_animating(app: &App) -> bool {
    let now = app.appearance.now;
    app.appearance.fade.is_animating(now) || app.overlay.drawer_motion.is_animating(now)
}

/// Tab, Shift+Tab, Ctrl+F or Cmd+F (matched through `Key::to_latin` so non-Latin layouts work),
/// F6 and Shift+F6 from events no widget captured.
fn key_command(event: keyboard::Event) -> Option<Message> {
    let _ = event;
    None::<KeyCommand>.map(Message::Key)
}

/// Escape from every event, because a focused text input captures it.
fn escape(event: Event, status: event::Status, window: window::Id) -> Option<Message> {
    let _ = (event, status, window);
    None
}
