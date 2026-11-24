//! iced interface: application state, messages, update, view and subscriptions, the custom
//! widgets, the panes built from stock widgets, loading, localization, theming, and the browser
//! host bridge.
//!
//! One state value (`state::App`) owns everything the view reads; widgets keep only transient
//! interaction state (scroll offsets, hover, drags, focus) in their iced tree. The filtered
//! result IDs live once in `state::DataState::results` and widgets borrow them.

mod context;
mod handoff;
#[cfg(all(feature = "web", target_family = "wasm"))]
pub(crate) mod host;
mod i18n;
mod ids;
mod layout;
mod links;
mod load;
mod message;
mod panes;
mod run;
#[cfg(all(test, feature = "desktop"))]
mod simulator_tests;
mod state;
mod subscription;
mod theme;
mod update;
mod view;
mod view_model;
mod widget;

#[cfg(feature = "desktop")]
pub use run::run;
#[cfg(all(feature = "web", target_family = "wasm"))]
pub(crate) use run::run_web;
