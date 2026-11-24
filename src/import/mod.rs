//! Vanilla EU5 installation importer: the location bundle from map, common and English
//! localization files, and the per-language packs that pair with it.

mod colors;
mod game;
mod hierarchy;
mod labels;
mod map_definition;
mod packs;
mod population;
mod ports;
mod progress;
mod records;
mod river_defines;
mod rivers;
mod templates;

pub use game::import_game;
pub(crate) use labels::record_use;
pub use packs::{
    load_paired_pack, pack_path, reindex_language_packs, selected_languages, write_language_packs,
};
pub use progress::{ImportProgress, ImportStage};
