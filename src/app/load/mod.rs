//! Loading: the dataset and index (a background thread on desktop, a decode after the first
//! painted frame in the browser), desktop retry and rebuild, language packs, and font packs.
//! Both targets expose the same `start`, `language_pack` and `font_pack` functions, each
//! returning a `Task` that reports through `LoadMessage`.

#[cfg(feature = "desktop")]
mod desktop;
#[cfg(feature = "desktop")]
mod external;
mod fonts;
mod packs;
mod prepare;
#[cfg(feature = "desktop")]
mod progress;
#[cfg(all(feature = "web", target_family = "wasm"))]
mod web;

#[cfg(feature = "desktop")]
pub(crate) use desktop::{font_pack, language_pack, start};
pub(crate) use fonts::embedded_faces;
#[cfg(feature = "desktop")]
pub(crate) use fonts::embedded_pack;
pub(crate) use packs::validate_pack_bytes;
pub(crate) use prepare::{LoadedData, prepare};
#[cfg(all(feature = "web", target_family = "wasm"))]
pub(crate) use web::{font_pack, language_pack, start};
