//! Field identities of the view-state grammar shared by the decoder and the encoder: names, the
//! protocol table order, duplicate tracking bits, and the item rules of each list field.

use super::limits::facet_field;
use crate::app::view_model::FacetId;
use crate::model::LocationKind;

/// One field of the grammar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Field {
    /// `q`: search text.
    Search,
    /// A facet list; its name comes from `facet_field`.
    Facet(FacetId),
    /// `fo`: food-producing raw materials only.
    FoodOnly,
    /// `c`: exact map colour.
    Rgb,
    /// `hr`: harbor suitability range.
    HarborRange,
    /// `mx`: first movement component range.
    MovementX,
    /// `my`: second movement component range.
    MovementY,
    /// `si`: impassable locations shown.
    ShowImpassable,
    /// `s`: sort.
    Sort,
    /// `l`: selected location key.
    Selected,
}

/// Every field in protocol table order, which is the canonical output order. It differs from the
/// rail order of `FacetId::ALL`, so it is spelled out here.
pub(super) const PROTOCOL_ORDER: [Field; 28] = [
    Field::Search,
    Field::Facet(FacetId::Kind),
    Field::Facet(FacetId::Topography),
    Field::Facet(FacetId::Vegetation),
    Field::Facet(FacetId::Climate),
    Field::Facet(FacetId::Continent),
    Field::Facet(FacetId::Subcontinent),
    Field::Facet(FacetId::Region),
    Field::Facet(FacetId::Area),
    Field::Facet(FacetId::Province),
    Field::Facet(FacetId::Religion),
    Field::Facet(FacetId::Culture),
    Field::Facet(FacetId::RawMaterial),
    Field::FoodOnly,
    Field::Facet(FacetId::Modifier),
    Field::Rgb,
    Field::Facet(FacetId::Coastal),
    Field::Facet(FacetId::RiverPresence),
    Field::Facet(FacetId::RiverLevelMin),
    Field::Facet(FacetId::RiverLevelMax),
    Field::Facet(FacetId::HarborPresence),
    Field::HarborRange,
    Field::Facet(FacetId::MovementPresence),
    Field::MovementX,
    Field::MovementY,
    Field::ShowImpassable,
    Field::Sort,
    Field::Selected,
];

impl Field {
    /// Field for a name; unknown names are `None`, which the decoder skips for forward
    /// compatibility.
    #[must_use]
    pub(super) fn from_name(name: &str) -> Option<Self> {
        match name {
            "q" => Some(Self::Search),
            "fo" => Some(Self::FoodOnly),
            "c" => Some(Self::Rgb),
            "hr" => Some(Self::HarborRange),
            "mx" => Some(Self::MovementX),
            "my" => Some(Self::MovementY),
            "si" => Some(Self::ShowImpassable),
            "s" => Some(Self::Sort),
            "l" => Some(Self::Selected),
            other => FacetId::ALL
                .into_iter()
                .find(|facet| facet_field(*facet) == other)
                .map(Self::Facet),
        }
    }

    /// Name written before the `.`.
    #[must_use]
    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::Search => "q",
            Self::Facet(facet) => facet_field(facet),
            Self::FoodOnly => "fo",
            Self::Rgb => "c",
            Self::HarborRange => "hr",
            Self::MovementX => "mx",
            Self::MovementY => "my",
            Self::ShowImpassable => "si",
            Self::Sort => "s",
            Self::Selected => "l",
        }
    }

    /// Distinct bit per field, so the decoder can ignore repeats with one `u32`.
    #[must_use]
    pub(super) fn bit(self) -> u32 {
        let position = match self {
            Self::Search => 0,
            // Facet indices are 0 to 18, so facets take bits 1 to 19.
            Self::Facet(facet) => u32::try_from(facet.index()).map_or(u32::MAX, |index| index + 1),
            Self::FoodOnly => 20,
            Self::Rgb => 21,
            Self::HarborRange => 22,
            Self::MovementX => 23,
            Self::MovementY => 24,
            Self::ShowImpassable => 25,
            Self::Sort => 26,
            Self::Selected => 27,
        };
        1_u32.checked_shl(position).unwrap_or(0)
    }
}

/// Which items a list field accepts before resolution against the dataset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ItemRule {
    /// A fixed vocabulary (kinds or presence words); the lone `!` is not allowed.
    Words(&'static [&'static str]),
    /// Dataset symbol keys; `missing` says whether the lone `!` is allowed.
    Keys {
        /// Whether the facet has a missing-value option.
        missing: bool,
    },
    /// River bonus tier digits, 1 to 255 without leading zeros.
    Tier,
}

/// Kind words in `LocationKind` order.
const KIND_WORDS: [&str; 5] = ["land", "sea", "lake", "impassable", "unknown"];
/// Coastal words.
const COASTAL_WORDS: [&str; 2] = ["yes", "no"];
/// Presence words of the river, harbor and movement facets.
const PRESENCE_WORDS: [&str; 2] = ["present", "missing"];

/// Item rule of a facet list.
#[must_use]
pub(super) const fn item_rule(facet: FacetId) -> ItemRule {
    match facet {
        FacetId::Kind => ItemRule::Words(&KIND_WORDS),
        FacetId::Coastal => ItemRule::Words(&COASTAL_WORDS),
        FacetId::RiverPresence | FacetId::HarborPresence | FacetId::MovementPresence => {
            ItemRule::Words(&PRESENCE_WORDS)
        }
        FacetId::RiverLevelMin | FacetId::RiverLevelMax => ItemRule::Tier,
        other => ItemRule::Keys {
            missing: other.has_missing_option(),
        },
    }
}

/// Wire word of a kind.
#[must_use]
pub(super) const fn kind_word(kind: LocationKind) -> &'static str {
    match kind {
        LocationKind::Land => "land",
        LocationKind::Sea => "sea",
        LocationKind::Lake => "lake",
        LocationKind::Impassable => "impassable",
        LocationKind::Unknown => "unknown",
    }
}

/// Wire word of a coastal state.
#[must_use]
pub(super) const fn coastal_word(coastal: bool) -> &'static str {
    if coastal { "yes" } else { "no" }
}

/// Wire word of a presence state.
#[must_use]
pub(super) const fn presence_word(present: bool) -> &'static str {
    if present { "present" } else { "missing" }
}
