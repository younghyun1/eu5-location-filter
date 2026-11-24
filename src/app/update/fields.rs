//! Validation of the rail text fields, and resetting them from an applied filter set.

use crate::app::i18n::Text;
use crate::app::state::TextFields;
use crate::app::view_model::{Bound, RangeField};
use crate::filter::FilterSet;
use crate::model::MapColor;

/// Parses exact-colour text: empty clears the filter, anything `MapColor::parse` rejects is
/// reported with the field's inline error key.
pub(super) fn parse_rgb(text: &str) -> Result<Option<MapColor>, Text> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    match MapColor::parse(trimmed) {
        Ok(color) => Ok(Some(color)),
        Err(_rejected) => Err(Text::ValidationRgb),
    }
}

/// Parses range bound text: empty clears the bound. Text that is not a number and a number
/// that is not finite get different messages, which `filter::parse_optional_number` cannot
/// tell apart, so the parse is local.
pub(super) fn parse_bound(text: &str) -> Result<Option<f32>, Text> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    match trimmed.parse::<f32>() {
        Ok(value) if value.is_finite() => Ok(Some(value)),
        Ok(_) => Err(Text::ValidationNumberFinite),
        Err(_) => Err(Text::ValidationNumber),
    }
}

/// Shortens `text` to at most `max_bytes` bytes without splitting a character.
pub(super) fn truncate_to(text: &mut String, max_bytes: usize) {
    if text.len() <= max_bytes {
        return;
    }
    let mut end = max_bytes;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    text.truncate(end);
}

/// Rewrites the colour and range fields to show an applied filter set, clearing their errors;
/// used when a view state replaces the filters from outside the rail.
pub(super) fn sync_from_filters(fields: &mut TextFields, filters: &FilterSet) {
    fields.rgb.text = match filters.rgb {
        Some(color) => color.hex(),
        None => String::new(),
    };
    fields.rgb.error = None;
    for field in RangeField::ALL {
        let range = field.get(filters);
        for (bound, value) in [(Bound::Min, range.min), (Bound::Max, range.max)] {
            let buffer = fields.range_mut(field, bound);
            buffer.text = match value {
                Some(value) => value.to_string(),
                None => String::new(),
            };
            buffer.error = None;
        }
    }
}
