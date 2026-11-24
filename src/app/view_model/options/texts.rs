//! Catalog keys for enum values, shared by option lists, cells and the detail pane.

use crate::app::i18n::Text;
use crate::model::LocationKind;

/// Catalog text of a location kind (`option.kind.*`).
#[must_use]
pub(in crate::app::view_model) const fn kind_text(kind: LocationKind) -> Text {
    match kind {
        LocationKind::Land => Text::OptionKindLand,
        LocationKind::Sea => Text::OptionKindSea,
        LocationKind::Lake => Text::OptionKindLake,
        LocationKind::Impassable => Text::OptionKindImpassable,
        LocationKind::Unknown => Text::OptionKindUnknown,
    }
}

/// Catalog text of a river tier name (`river.tier.*`); tiers above the known five read
/// `river.tier.unknown`.
#[must_use]
pub(in crate::app::view_model) const fn river_tier_text(level: u8) -> Text {
    match level {
        1 => Text::RiverTierBrook,
        2 => Text::RiverTierStream,
        3 => Text::RiverTierRiver,
        4 => Text::RiverTierLargeRiver,
        5 => Text::RiverTierMajorRiver,
        _ => Text::RiverTierUnknown,
    }
}
