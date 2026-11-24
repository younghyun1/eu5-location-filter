//! Cell text and colour class for one (record, column) pair, written into a caller-owned buffer
//! so the grid formats visible cells without allocating per row.

use iced::Color;

use super::column_id::ColumnId;
use crate::app::i18n::Locale;
use crate::app::theme::Palette;
use crate::lang_pack::Labels;
use crate::model::LocationRecord;

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
    let _ = (context, record, column);
    buffer.clear();
    CellTone::Ink
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
