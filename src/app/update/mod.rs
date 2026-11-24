//! Update: one dispatcher and one helper module per message group. Only update helpers write
//! application state; every helper returns the follow-up task.

mod dispatch;
mod fields;
mod filters;
mod focus;
mod host;
mod keyboard;
mod load;
mod packs;
mod panes;
mod settings;
mod surfaces;
mod table;
#[cfg(test)]
mod tests;

pub(crate) use dispatch::update;
#[cfg(all(feature = "web", target_family = "wasm"))]
pub(crate) use host::apply_host_snapshot;
