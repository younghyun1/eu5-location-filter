//! Per-language game text packs: localized labels, search strings, and label-ordered sorts.
//!
//! English stays in the main bundles: its labels live in `StoredDataset.dictionary` and
//! `StoredDataset.localizations`, and its search and label-dependent sort data live in the
//! English [`StoredLanguageSection`] inside the filter index. Each other language ships as one
//! [`StoredLanguagePack`] file that pairs with exactly one dataset build. Symbols keep their
//! dataset [`SymbolId`](crate::model::SymbolId), so a pack only adds text and never renumbers
//! records.

#[cfg(feature = "desktop")]
mod builder;
mod codec;
#[cfg(all(test, not(target_family = "wasm")))]
mod codec_tests;
mod fields;
#[cfg(feature = "desktop")]
mod files;
mod language;
mod lookup;
mod schema;
mod validate;
#[cfg(test)]
mod validate_tests;

#[cfg(feature = "desktop")]
pub use builder::{build_pack, import_pack, rebuild_pack, requested_keys};
pub use codec::{decode_pack, dictionary_digest, pack_file_name, validate_pack};
#[cfg(feature = "desktop")]
pub use codec::{encode_pack, load_pack, write_pack};
pub use fields::{
    LABEL_SORT_FIELDS, LANGUAGE_INDEPENDENT_SORT_FIELDS, LabelField, is_label_dependent,
};
pub use language::GameLanguage;
pub use lookup::{Labels, ResolvedText, ValidatedLanguagePack, resolve_text};
pub use schema::{
    MAX_PACK_COMPRESSED_SIZE, MAX_PACK_DECOMPRESSED_SIZE, MAX_PACK_STRING_BYTES, PACK_DIRECTORY,
    PACK_FORMAT_VERSION, PACK_MAGIC, PackLabel, PackStringId, StoredLanguagePack,
    StoredLanguageSection, StoredOrder, TrigramPosting,
};
pub(crate) use validate::validate_paired;
