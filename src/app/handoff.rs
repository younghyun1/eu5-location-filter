//! Take-once transfer of values that cannot be cloned through messages that must be cloned.

use std::fmt;
use std::sync::{Arc, Mutex};

/// Carries a value that is not `Clone` (a loaded filter engine, a validated language pack)
/// inside a `Message`, which iced requires to be `Clone + Send`. Clones share one slot; the first
/// `take` receives the value and every later one receives `None`, so a message delivered twice
/// cannot install the same engine twice.
pub(crate) struct Handoff<T>(Arc<Mutex<Option<T>>>);

impl<T> Handoff<T> {
    /// Wraps a value for one later `take`.
    #[must_use]
    pub(crate) fn new(value: T) -> Self {
        Self(Arc::new(Mutex::new(Some(value))))
    }

    /// Removes the value; a poisoned lock still yields it, because the slot holds no invariant
    /// that a panic elsewhere could break.
    #[must_use]
    pub(crate) fn take(&self) -> Option<T> {
        match self.0.lock() {
            Ok(mut slot) => slot.take(),
            Err(poisoned) => poisoned.into_inner().take(),
        }
    }
}

impl<T> Clone for Handoff<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T> fmt::Debug for Handoff<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Handoff(..)")
    }
}
