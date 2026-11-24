//! Cell text and colour class for one (record, column) pair, written into a caller-owned buffer
//! so the grid formats visible cells without allocating per row.

use iced::Color;

use super::column_id::ColumnId;
use super::options::kind_text;
use crate::app::i18n::{Locale, Text, format_into, push_fixed, push_grouped, t};
use crate::app::theme::Palette;
use crate::lang_pack::Labels;
use crate::model::{
    Dataset, LocationRecord, MapColor, SymbolId, is_food_producing, is_gold_or_silver,
};

/// Logical cell alignment; mirrors in right-to-left layouts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CellAlign {
    /// Inline start.
    Start,
    /// Inline end, for numbers.
    End,
}

/// How a column's cells are drawn.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CellFont {
    /// Sans Regular.
    Sans,
    /// Sans Medium, the name column.
    SansMedium,
    /// Mono Regular: identifiers, numbers, RGB.
    Mono,
    /// Colour swatch followed by the hex value in Mono.
    Swatch,
}

/// Colour class of a cell or detail value.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum CellTone {
    /// Primary text.
    #[default]
    Ink,
    /// Secondary text.
    #[cfg_attr(not(test), allow(dead_code))]
    Muted,
    /// Food-producing raw material.
    Food,
    /// Gold or silver raw material.
    Precious,
    /// River bonus cell.
    River,
}

/// Everything a cell needs besides the record: game text for the active language and the
/// interface locale for catalog values such as `option.yes` and `value.none`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RowContext<'a> {
    /// Names and labels with English fallback.
    pub(crate) labels: Labels<'a>,
    /// Interface locale.
    pub(crate) locale: Locale,
}

/// Clears `buffer` and writes the text of `column` for `record`: borrowed labels are copied in,
/// numbers use invariant formatting, missing values write `value.none`. For the colour column
/// it writes the hex value; the grid draws the swatch from `record.color`.
pub(crate) fn write_cell(
    buffer: &mut String,
    context: &RowContext<'_>,
    record: &LocationRecord,
    column: ColumnId,
) -> CellTone {
    buffer.clear();
    let labels = context.labels;
    let locale = context.locale;
    let none = t(locale, Text::ValueNone);
    let symbol = |id: Option<SymbolId>| id.and_then(|id| labels.label(id)).unwrap_or(none);
    let hierarchy = &record.hierarchy;
    let text: &str = match column {
        ColumnId::Color => {
            push_hex(buffer, record.color);
            return CellTone::Ink;
        }
        ColumnId::Name => labels.name(record.id).unwrap_or(none),
        ColumnId::Identifier => labels.dataset().symbol(record.key).unwrap_or(none),
        ColumnId::Kind => t(locale, kind_text(record.kind)),
        ColumnId::Topography => symbol(Some(record.topography)),
        ColumnId::Vegetation => symbol(record.vegetation),
        ColumnId::Climate => symbol(record.climate),
        ColumnId::Continent => symbol(Some(hierarchy.continent)),
        ColumnId::Subcontinent => symbol(Some(hierarchy.subcontinent)),
        ColumnId::Region => symbol(Some(hierarchy.region)),
        ColumnId::Area => symbol(Some(hierarchy.area)),
        ColumnId::Province => symbol(Some(hierarchy.province)),
        ColumnId::Religion => symbol(record.religion),
        ColumnId::Culture => symbol(record.culture),
        ColumnId::RawMaterial => {
            buffer.push_str(symbol(record.raw_material));
            return material_tone(labels.dataset(), record.raw_material);
        }
        ColumnId::Modifier => symbol(record.modifier),
        ColumnId::Rgb => {
            let [red, green, blue] = record.color.components();
            let (red, green, blue) = (
                Digits::new(u64::from(red)),
                Digits::new(u64::from(green)),
                Digits::new(u64::from(blue)),
            );
            format_into(
                buffer,
                t(locale, Text::FormatRgb),
                &[
                    ("red", red.as_str()),
                    ("green", green.as_str()),
                    ("blue", blue.as_str()),
                ],
            );
            return CellTone::Ink;
        }
        ColumnId::Coastal => t(locale, yes_no(record.coastal)),
        ColumnId::RiverPresence => t(locale, present_missing(record.river.is_some())),
        ColumnId::RiverLevel => {
            return match record.river {
                Some(river) => {
                    let level = Digits::new(u64::from(river.level.0));
                    let percent = Digits::new(u64::from(river.level.0).saturating_mul(10));
                    format_into(
                        buffer,
                        t(locale, Text::FormatRiverCell),
                        &[("level", level.as_str()), ("percent", percent.as_str())],
                    );
                    CellTone::River
                }
                None => {
                    buffer.push_str(none);
                    CellTone::Ink
                }
            };
        }
        ColumnId::Harbor => {
            push_optional_fixed(buffer, record.harbor_suitability, none);
            return CellTone::Ink;
        }
        ColumnId::MovementPresence => t(
            locale,
            present_missing(record.movement_assistance.is_some()),
        ),
        ColumnId::MovementX => {
            push_optional_fixed(
                buffer,
                record.movement_assistance.map(|value| value[0]),
                none,
            );
            return CellTone::Ink;
        }
        ColumnId::MovementY => {
            push_optional_fixed(
                buffer,
                record.movement_assistance.map(|value| value[1]),
                none,
            );
            return CellTone::Ink;
        }
        ColumnId::StaticCapacity => {
            push_optional_grouped(
                buffer,
                record.static_population_capacity.map(|value| value.total.0),
                none,
            );
            return CellTone::Ink;
        }
        ColumnId::EquatorCapacity => {
            push_optional_grouped(
                buffer,
                record
                    .static_population_capacity
                    .map(|value| value.equator.0),
                none,
            );
            return CellTone::Ink;
        }
    };
    buffer.push_str(text);
    CellTone::Ink
}

/// Tone of a raw-material value: food-producing goods, gold and silver, or neither.
#[must_use]
pub(super) fn material_tone(dataset: &Dataset, material: Option<SymbolId>) -> CellTone {
    match material.and_then(|symbol| dataset.symbol(symbol)) {
        Some(key) if is_food_producing(key) => CellTone::Food,
        Some(key) if is_gold_or_silver(key) => CellTone::Precious,
        _ => CellTone::Ink,
    }
}

/// Catalog text of a yes or no value.
#[must_use]
pub(super) const fn yes_no(value: bool) -> Text {
    if value {
        Text::OptionYes
    } else {
        Text::OptionNo
    }
}

/// Catalog text of a present or missing value.
#[must_use]
pub(super) const fn present_missing(value: bool) -> Text {
    if value {
        Text::OptionPresent
    } else {
        Text::OptionMissing
    }
}

/// Appends `value` with two decimals, or `none` when absent.
pub(super) fn push_optional_fixed(buffer: &mut String, value: Option<f32>, none: &str) {
    match value {
        Some(value) => push_fixed(buffer, value, 2),
        None => buffer.push_str(none),
    }
}

/// Appends a population with digit grouping, or `none` when absent.
fn push_optional_grouped(buffer: &mut String, value: Option<u32>, none: &str) {
    match value {
        Some(value) => push_grouped(buffer, u64::from(value)),
        None => buffer.push_str(none),
    }
}

/// Appends `#RRGGBB` in uppercase without allocating.
fn push_hex(buffer: &mut String, color: MapColor) {
    buffer.push('#');
    for position in (0..6u32).rev() {
        let nibble = (color.0 >> (position * 4)) & 0xF;
        if let Some(digit) = char::from_digit(nibble, 16) {
            buffer.push(digit.to_ascii_uppercase());
        }
    }
}

/// Decimal digits of an unsigned number in a stack buffer, for placeholder arguments that must
/// not allocate per cell.
#[derive(Clone, Copy, Debug)]
pub(super) struct Digits {
    /// Digits right-aligned in the buffer.
    bytes: [u8; 20],
    /// Index of the first digit.
    start: usize,
}

impl Digits {
    /// Formats `value` in base ten.
    #[must_use]
    pub(super) fn new(mut value: u64) -> Self {
        let mut bytes = [b'0'; 20];
        let mut start = bytes.len();
        loop {
            start = start.saturating_sub(1);
            let digit = u8::try_from(value % 10).unwrap_or(0);
            if let Some(slot) = bytes.get_mut(start) {
                *slot = b'0'.saturating_add(digit);
            }
            value /= 10;
            if value == 0 || start == 0 {
                break;
            }
        }
        Self { bytes, start }
    }

    /// The digits as text.
    #[must_use]
    pub(super) fn as_str(&self) -> &str {
        self.bytes
            .get(self.start..)
            .and_then(|digits| std::str::from_utf8(digits).ok())
            .unwrap_or("")
    }
}

/// Palette colour of a tone.
#[must_use]
pub(crate) fn tone_color(palette: Palette, tone: CellTone) -> Color {
    match tone {
        CellTone::Ink => palette.ink,
        CellTone::Muted => palette.ink_muted,
        CellTone::Food => palette.food(),
        CellTone::Precious => palette.precious(),
        CellTone::River => palette.river_cell(),
    }
}
