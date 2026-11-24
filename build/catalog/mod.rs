//! Interface catalog generation for `build.rs`: parses `assets/i18n/*.txt`, checks every locale
//! against English, and writes the generated `Text` enum and per-locale tables.

mod check;
mod generate;
mod locales;
mod model;
mod parse;
mod render;

pub use generate::generate_catalogs;
pub use locales::CATALOG_FILE;
