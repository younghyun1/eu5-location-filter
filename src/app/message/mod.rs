//! Every interaction the application handles, grouped by the update helper that owns it.
//!
//! iced requires messages to be `Clone + Send + 'static` on every target, so values that cannot
//! be cloned travel in a `Handoff` and errors travel as `Arc<AppError>`. Option and column
//! identities are typed enums, never display strings.

mod host;
mod load;
mod interaction;

pub(crate) use host::{HostMessage, HostSnapshot};
pub(crate) use load::LoadMessage;
pub(crate) use interaction::{
    ColumnMessage, FilterMessage, GridMessage, KeyCommand, Message, OverlayMessage, PaneMessage,
    SelectionSource, SettingsMessage,
};
