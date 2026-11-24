//! Font pack generator: builds the embedded IBM Plex faces and the lazily fetched script packs
//! from local source fonts.
//!
//! Each output is described in `spec`. The module collects the codepoints an output must cover
//! from the interface catalogs and the language packs (`codepoints`, `sources`), asks the one
//! subsetting function (`subset`) for a reduced font, and then reads the result's `cmap` back
//! to prove that every required codepoint survived (`coverage`, `cmap`). A manifest with the
//! source digests is written last (`manifest`). Source fonts, licences and the command are
//! documented in `assets/fonts/README.md`.

mod cmap;
mod codepoints;
mod coverage;
mod manifest;
mod report;
mod run;
mod sha256;
mod sources;
mod spec;
mod subset;

pub use report::{FontOutputReport, FontPackReport};
pub use run::{FontPackArgs, run};
