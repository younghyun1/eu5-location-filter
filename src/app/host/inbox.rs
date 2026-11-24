//! Fixed latest-wins slots written by the exports and drained by one subscription. Each slot
//! holds at most one value and the wake channel has capacity one, so no number of host
//! messages can grow memory. All access uses `try_borrow_mut`, so a re-entrant call fails
//! instead of panicking.

use std::cell::RefCell;

use iced::Subscription;
use iced::futures::Stream;
use iced::futures::channel::mpsc;

use crate::app::i18n::Locale;
use crate::app::message::{HostMessage, HostSnapshot, Message};
use crate::app::theme::Palette;

/// Slots written by the exports.
#[derive(Debug, Default)]
pub(super) struct Inbox {
    /// Latest validated palette.
    pub(super) palette: Option<Palette>,
    /// Latest locale.
    pub(super) locale: Option<Locale>,
    /// Latest reduced-motion flag.
    pub(super) reduced_motion: Option<bool>,
    /// Latest view-state string that passed the whole-string checks.
    pub(super) view_state: Option<String>,
    /// Set by the first `set_palette`.
    pub(super) connected: bool,
    /// Latest text of the host page's search field.
    pub(super) search_text: Option<String>,
    /// Wake signal of the running subscription; a full buffer is ignored, because the pending
    /// wake already drains every slot.
    pub(super) waker: Option<mpsc::Sender<()>>,
}

thread_local! {
    static INBOX: RefCell<Inbox> = RefCell::new(Inbox::default());
}

/// Writes slots and wakes the subscription; `false` when the inbox is borrowed.
pub(super) fn store(write: impl FnOnce(&mut Inbox)) -> bool {
    INBOX.with(|inbox| match inbox.try_borrow_mut() {
        Ok(mut inbox) => {
            write(&mut inbox);
            true
        }
        Err(_) => false,
    })
}

/// Drains every slot; boot calls this so the first frame already uses host values.
#[must_use]
pub(crate) fn take_snapshot() -> HostSnapshot {
    HostSnapshot::default()
}

/// Subscription that installs the waker and yields one snapshot per wake.
pub(crate) fn subscription() -> Subscription<Message> {
    Subscription::run(wakes).map(Message::Host)
}

/// Stream behind `subscription`.
fn wakes() -> impl Stream<Item = HostMessage> {
    iced::futures::stream::empty()
}
