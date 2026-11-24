//! Update: one dispatcher and one helper module per message group. Only update helpers write
//! application state; every helper returns the follow-up task.

mod dispatch;
mod filters;
mod host;
mod keyboard;
mod load;
mod panes;
mod settings;
mod table;

pub(crate) use dispatch::update;
#[cfg(all(feature = "web", target_family = "wasm"))]
pub(crate) use host::apply_host_snapshot;
