//! Serialized pack and language-section types, plus envelope constants and limits.

use bitcode::{Decode, Encode};

use super::GameLanguage;
use crate::filter::SortField;
use crate::model::{ImportDiagnostic, LocationId, SymbolId};

/// Schema version of [`StoredLanguagePack`]; bumps whenever [`StoredLanguageSection`] changes.
pub const PACK_FORMAT_VERSION: u16 = 1;
/// Envelope magic and envelope version, followed by the payload length as a little-endian `u64`.
pub const PACK_MAGIC: &[u8; 8] = b"EU5LNG\0\x01";
/// Largest accepted compressed pack, checked before decompression starts.
pub const MAX_PACK_COMPRESSED_SIZE: u64 = 16 * 1024 * 1024;
/// Largest accepted decompressed envelope, so a hostile frame cannot grow memory without limit.
pub const MAX_PACK_DECOMPRESSED_SIZE: u64 = 64 * 1024 * 1024;
/// Longest accepted localized string in bytes; real labels are far shorter.
pub const MAX_PACK_STRING_BYTES: usize = 1024;
/// Directory, relative to the bundle directory, that holds pack files.
pub const PACK_DIRECTORY: &str = "lang";

/// Index into [`StoredLanguagePack::strings`].
#[derive(Clone, Copy, Debug, Decode, Encode, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PackStringId(pub u32);

/// Localized label for one dataset symbol whose text differs from English.
#[derive(Clone, Copy, Debug, Decode, Encode, Eq, PartialEq)]
pub struct PackLabel {
    /// Dataset symbol of the internal identifier.
    pub key: SymbolId,
    /// Localized text.
    pub value: PackStringId,
}

/// Increasing unique location IDs whose search text contains one byte trigram.
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct TrigramPosting {
    /// Three consecutive bytes of the folded search text.
    pub key: [u8; 3],
    /// Locations containing the trigram.
    pub ids: Vec<LocationId>,
}

/// One column's ascending and descending permutations with their inverse ranks.
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct StoredOrder {
    /// Column represented by both permutations.
    pub field: SortField,
    /// Ascending IDs with nulls last and ID tie-breaks.
    pub ascending: Vec<LocationId>,
    /// Descending IDs with nulls last and ID tie-breaks.
    pub descending: Vec<LocationId>,
    /// Inverse of `ascending`, indexed by location ID.
    pub ascending_ranks: Vec<u32>,
    /// Inverse of `descending`, indexed by location ID.
    pub descending_ranks: Vec<u32>,
}

/// Language-dependent query data shared by the English index section and every pack.
#[derive(Clone, Debug, Default, Decode, Encode, Eq, PartialEq)]
pub struct StoredLanguageSection {
    /// Per location: the folded localized name when it differs from the folded English name,
    /// then the folded English name and the folded identifier, joined by NUL.
    pub searchable: Vec<String>,
    /// Trigram postings over `searchable`, strictly increasing by key.
    pub trigrams: Vec<TrigramPosting>,
    /// Orders for [`LABEL_SORT_FIELDS`](super::LABEL_SORT_FIELDS), in that order.
    pub orders: Vec<StoredOrder>,
}

/// Payload of `assets/lang/<code>.<build>.bitcode.zst`.
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct StoredLanguagePack {
    /// Must equal [`PACK_FORMAT_VERSION`].
    pub format_version: u16,
    /// EU5 Steam application ID.
    pub app_id: u32,
    /// Build ID of the paired dataset.
    pub build_id: u64,
    /// Record count of the paired dataset.
    pub location_count: u32,
    /// Dictionary length of the paired dataset; bounds every `SymbolId` in the pack.
    pub symbol_count: u32,
    /// [`dictionary_digest`](super::dictionary_digest) of the paired dataset, so two imports
    /// that share a build ID but differ in content cannot pair.
    pub dictionary_digest: u64,
    /// Language of every string in the pack; never English.
    pub language: GameLanguage,
    /// Distinct localized strings.
    pub strings: Vec<String>,
    /// Localized name per location ID; `None` where it equals the English name.
    pub location_names: Vec<Option<PackStringId>>,
    /// Labels strictly increasing by `key`; absent symbols use the English label.
    pub labels: Vec<PackLabel>,
    /// Non-fatal findings, such as unresolved references or missing labels.
    pub diagnostics: Vec<ImportDiagnostic>,
    /// Search and label-sort data resolved with this pack's text.
    pub section: StoredLanguageSection,
}
