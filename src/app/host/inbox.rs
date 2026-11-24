//! Fixed latest-wins slots written by the exports and drained by one subscription. Each slot
//! holds at most one value and the wake channel has capacity one, so no number of host
//! messages can grow memory. All access uses `try_borrow_mut`, so a re-entrant call fails
//! instead of panicking.

use std::cell::RefCell;

use iced::Subscription;
use iced::futures::channel::mpsc;
use iced::futures::{Stream, StreamExt};

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

impl Inbox {
    /// Whether any slot holds a value that has not been drained.
    fn has_pending(&self) -> bool {
        self.palette.is_some()
            || self.locale.is_some()
            || self.reduced_motion.is_some()
            || self.view_state.is_some()
            || self.search_text.is_some()
    }
}

thread_local! {
    static INBOX: RefCell<Inbox> = RefCell::new(Inbox::default());
}

/// Writes slots and wakes the subscription; `false` when the inbox is borrowed.
pub(super) fn store(write: impl FnOnce(&mut Inbox)) -> bool {
    INBOX.with(|inbox| match inbox.try_borrow_mut() {
        Ok(mut inbox) => {
            write(&mut inbox);
            // `try_send` only schedules the receiving task, so it cannot re-enter this borrow.
            // A full buffer already holds a wake and a closed channel has no subscription to wake;
            // both are fine to ignore.
            if let Some(waker) = inbox.waker.as_mut() {
                let _ = waker.try_send(());
            }
            true
        }
        Err(_) => false,
    })
}

/// Drains every slot; boot calls this so the first frame already uses host values.
#[must_use]
pub(crate) fn take_snapshot() -> HostSnapshot {
    INBOX.with(|inbox| match inbox.try_borrow_mut() {
        Ok(mut inbox) => HostSnapshot {
            palette: inbox.palette.take(),
            locale: inbox.locale.take(),
            reduced_motion: inbox.reduced_motion.take(),
            view_state: inbox.view_state.take(),
            // Connection is a state, not an event, so it stays set across drains.
            connected: inbox.connected,
            search_text: inbox.search_text.take(),
        },
        // Only a re-entrant call can fail here; the values stay queued for the next wake.
        Err(_) => HostSnapshot::default(),
    })
}

/// Subscription that installs the waker and yields one snapshot per wake.
pub(crate) fn subscription() -> Subscription<Message> {
    Subscription::run(wakes).map(Message::Host)
}

/// Stream behind `subscription`. Installing a new sender replaces any earlier one, so a
/// restarted subscription takes over the wakes and the old channel closes. The stream ends only
/// if the sender is dropped, which happens only when a newer subscription replaced it.
fn wakes() -> impl Stream<Item = HostMessage> {
    let (mut sender, receiver) = mpsc::channel(1);
    INBOX.with(|inbox| {
        if let Ok(mut inbox) = inbox.try_borrow_mut() {
            // Exports may have run between boot's drain and this subscription starting, when no
            // waker existed; one initial wake drains whatever they stored.
            if inbox.has_pending() || inbox.connected {
                let _ = sender.try_send(());
            }
            inbox.waker = Some(sender);
        }
    });
    receiver.map(|()| HostMessage::Update(take_snapshot()))
}
