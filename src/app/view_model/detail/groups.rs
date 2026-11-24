//! The six field groups of the detail pane.

use super::model::{DetailField, DetailGroup};
use crate::app::i18n::{Locale, Text, format, push_grouped, t};
use crate::app::view_model::options::{kind_text, river_tier_text};
use crate::app::view_model::row::{
    CellTone, Digits, material_tone, present_missing, push_optional_fixed, yes_no,
};
use crate::lang_pack::Labels;
use crate::model::{Dataset, LocationRecord, SymbolId};

/// Inputs shared by the group builders.
pub(super) struct Fields<'a> {
    /// Names and labels for the active language.
    labels: Labels<'a>,
    /// Interface locale.
    locale: Locale,
    /// Dataset behind `labels`.
    dataset: &'a Dataset,
    /// Location shown.
    record: &'a LocationRecord,
    /// `value.none` for the locale.
    none: &'static str,
}

impl<'a> Fields<'a> {
    /// Bundles the inputs of one location.
    #[must_use]
    pub(super) fn new(
        labels: Labels<'a>,
        locale: Locale,
        record: &'a LocationRecord,
        none: &'static str,
    ) -> Self {
        Self {
            labels,
            locale,
            dataset: labels.dataset(),
            record,
            none,
        }
    }

    /// A plain value row.
    fn row(&self, label: Text, value: String) -> DetailField {
        DetailField {
            label,
            value,
            tone: CellTone::Ink,
            mono: false,
        }
    }

    /// A technical value row, laid out left to right in Mono.
    fn mono(&self, label: Text, value: String) -> DetailField {
        DetailField {
            mono: true,
            ..self.row(label, value)
        }
    }

    /// Game label of an optional symbol, or `value.none`.
    fn symbol(&self, symbol: Option<SymbolId>) -> String {
        symbol
            .and_then(|symbol| self.labels.label(symbol))
            .unwrap_or(self.none)
            .to_owned()
    }

    /// Catalog text as an owned value.
    fn text(&self, text: Text) -> String {
        t(self.locale, text).to_owned()
    }

    /// Type, topography, vegetation and climate.
    pub(super) fn classification(&self) -> DetailGroup {
        let record = self.record;
        DetailGroup {
            title: Text::DetailGroupClassification,
            fields: vec![
                self.row(Text::DetailFieldType, self.text(kind_text(record.kind))),
                self.row(
                    Text::DetailFieldTopography,
                    self.symbol(Some(record.topography)),
                ),
                self.row(Text::DetailFieldVegetation, self.symbol(record.vegetation)),
                self.row(Text::DetailFieldClimate, self.symbol(record.climate)),
            ],
            note: None,
        }
    }

    /// Religion and culture.
    pub(super) fn society(&self) -> DetailGroup {
        DetailGroup {
            title: Text::DetailGroupSociety,
            fields: vec![
                self.row(Text::DetailFieldReligion, self.symbol(self.record.religion)),
                self.row(Text::DetailFieldCulture, self.symbol(self.record.culture)),
            ],
            note: None,
        }
    }

    /// Raw material, with its food or precious tone, and modifier.
    pub(super) fn economy(&self) -> DetailGroup {
        let record = self.record;
        let material = DetailField {
            tone: material_tone(self.dataset, record.raw_material),
            ..self.row(
                Text::DetailFieldRawMaterial,
                self.symbol(record.raw_material),
            )
        };
        DetailGroup {
            title: Text::DetailGroupEconomy,
            fields: vec![
                material,
                self.row(Text::DetailFieldModifier, self.symbol(record.modifier)),
            ],
            note: None,
        }
    }

    /// Coastal state, connected sea, harbor suitability and movement assistance.
    pub(super) fn coast(&self) -> DetailGroup {
        let record = self.record;
        let sea = record
            .connected_sea
            .and_then(|id| self.dataset.location(id))
            .and_then(|sea| self.dataset.symbol(sea.key))
            .unwrap_or(self.none)
            .to_owned();
        let mut harbor = String::new();
        push_optional_fixed(&mut harbor, record.harbor_suitability, self.none);
        let movement = match record.movement_assistance {
            Some([x, y]) => {
                let mut x_text = String::new();
                let mut y_text = String::new();
                push_optional_fixed(&mut x_text, Some(x), self.none);
                push_optional_fixed(&mut y_text, Some(y), self.none);
                format(
                    t(self.locale, Text::FormatMovement),
                    &[("x", x_text.as_str()), ("y", y_text.as_str())],
                )
            }
            None => self.none.to_owned(),
        };
        DetailGroup {
            title: Text::DetailGroupCoast,
            fields: vec![
                self.row(Text::DetailFieldCoastal, self.text(yes_no(record.coastal))),
                self.mono(Text::DetailFieldConnectedSea, sea),
                self.mono(Text::DetailFieldHarbor, harbor),
                self.mono(Text::DetailFieldMovement, movement),
            ],
            note: None,
        }
    }

    /// River tier, and with a river its capacity bonus, source and confluence.
    pub(super) fn river(&self) -> DetailGroup {
        let Some(river) = self.record.river else {
            return DetailGroup {
                title: Text::DetailGroupRiver,
                fields: vec![self.row(Text::DetailFieldRiver, self.text(present_missing(false)))],
                note: None,
            };
        };
        let level = Digits::new(u64::from(river.level.0));
        let percent = Digits::new(u64::from(river.level.0).saturating_mul(10));
        let summary = format(
            t(self.locale, Text::FormatRiverDetail),
            &[
                ("tier", t(self.locale, river_tier_text(river.level.0))),
                ("level", level.as_str()),
            ],
        );
        let bonus = format(
            t(self.locale, Text::FormatPercentBonus),
            &[("percent", percent.as_str())],
        );
        DetailGroup {
            title: Text::DetailGroupRiver,
            fields: vec![
                self.row(Text::DetailFieldRiver, summary),
                self.mono(Text::DetailFieldRiverBonus, bonus),
                self.row(
                    Text::DetailFieldRiverSource,
                    self.text(yes_no(river.has_source)),
                ),
                self.row(
                    Text::DetailFieldRiverConfluence,
                    self.text(yes_no(river.has_confluence)),
                ),
            ],
            note: None,
        }
    }

    /// Static population capacity and its note.
    pub(super) fn capacity(&self) -> DetailGroup {
        let grouped = |amount: Option<u32>| match amount {
            Some(amount) => {
                let mut text = String::new();
                push_grouped(&mut text, u64::from(amount));
                text
            }
            None => self.none.to_owned(),
        };
        let capacity = self.record.static_population_capacity;
        let modifier = match capacity {
            Some(value) => {
                let signed = signed_hundredths(value.modifier_basis_points);
                format(
                    t(self.locale, Text::FormatSignedPercent),
                    &[("value", signed.as_str())],
                )
            }
            None => self.none.to_owned(),
        };
        DetailGroup {
            title: Text::DetailGroupCapacity,
            fields: vec![
                self.mono(
                    Text::DetailFieldCapacityTotal,
                    grouped(capacity.map(|value| value.total.0)),
                ),
                self.mono(
                    Text::DetailFieldCapacityVegetation,
                    grouped(capacity.map(|value| value.vegetation.0)),
                ),
                self.mono(
                    Text::DetailFieldCapacityEquator,
                    grouped(capacity.map(|value| value.equator.0)),
                ),
                self.mono(Text::DetailFieldCapacityModifier, modifier),
            ],
            note: Some(Text::DetailCapacityNote),
        }
    }
}

/// Basis points as a signed percentage with two decimals, such as `+1.50` or `-0.05`; exact
/// integer arithmetic keeps the text identical in every locale.
#[must_use]
fn signed_hundredths(basis_points: i16) -> String {
    let magnitude = i32::from(basis_points).unsigned_abs();
    let sign = if basis_points < 0 { '-' } else { '+' };
    let whole = Digits::new(u64::from(magnitude / 100));
    let fraction = magnitude % 100;
    format!(
        "{sign}{}.{}{}",
        whole.as_str(),
        fraction / 10,
        fraction % 10
    )
}
