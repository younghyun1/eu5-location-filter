//! Top-level update dispatch.

use iced::Task;

use super::{filters, host, keyboard, load, panes, settings, table};
use crate::app::message::{HostMessage, Message};
use crate::app::state::App;

/// Applies one message and returns its follow-up task. After every message the canonical
/// view-state string is reported to the browser host if it changed and reporting is enabled.
pub(crate) fn update(app: &mut App, message: Message) -> Task<Message> {
    let task = match message {
        Message::Filter(message) => filters::filter(app, message),
        Message::Grid(message) => table::grid(app, message),
        Message::Column(message) => table::column(app, message),
        Message::Pane(message) => panes::pane(app, message),
        Message::Overlay(message) => panes::overlay(app, message),
        Message::Settings(message) => settings::settings(app, message),
        Message::Load(message) => load::load(app, message),
        Message::Host(HostMessage::Update(snapshot)) => host::apply_host_snapshot(app, snapshot),
        Message::Key(command) => keyboard::key(app, command),
        Message::WindowResized(size) => panes::resized(app, size),
        Message::Frame(now) => settings::frame(app, now),
    };
    host::report_view_state(app);
    task
}
