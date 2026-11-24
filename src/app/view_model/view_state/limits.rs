//! Fixed bounds and field names of the view-state grammar (host protocol version 2).

use crate::app::view_model::FacetId;

/// Grammar version token at the start of every string.
pub(crate) const VIEW_STATE_VERSION: &str = "s1";
/// Longest accepted or reported string, in bytes (the charset is ASCII).
pub(crate) const VIEW_STATE_MAX_LEN: usize = 2_000;
/// Fields read per string; later ones are ignored.
pub(crate) const VIEW_STATE_MAX_FIELDS: usize = 64;
/// Items read per list; later ones are ignored.
pub(crate) const VIEW_STATE_MAX_ITEMS: usize = 1_000;
/// Longest decoded search text, in bytes.
pub(crate) const MAX_SEARCH_BYTES: usize = 256;
/// Longest decoded symbol or location key, in bytes.
pub(crate) const MAX_KEY_BYTES: usize = 128;
/// Longest number token, in characters.
pub(crate) const MAX_NUMBER_CHARS: usize = 48;

/// Field name of a facet in the string. Field order in the string is the protocol table order,
/// which differs from rail order; the encoder writes that order explicitly.
#[must_use]
pub(crate) const fn facet_field(facet: FacetId) -> &'static str {
    match facet {
        FacetId::Kind => "k",
        FacetId::Continent => "ct",
        FacetId::Subcontinent => "sc",
        FacetId::Region => "rg",
        FacetId::Area => "ar",
        FacetId::Province => "pv",
        FacetId::Topography => "tp",
        FacetId::Vegetation => "vg",
        FacetId::Climate => "cl",
        FacetId::Religion => "rl",
        FacetId::Culture => "cu",
        FacetId::RawMaterial => "rm",
        FacetId::Modifier => "md",
        FacetId::Coastal => "co",
        FacetId::RiverPresence => "rp",
        FacetId::RiverLevelMin => "lm",
        FacetId::RiverLevelMax => "lx",
        FacetId::HarborPresence => "hp",
        FacetId::MovementPresence => "mp",
    }
}
