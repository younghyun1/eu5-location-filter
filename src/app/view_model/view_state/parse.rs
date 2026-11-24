//! Value parsers for single fields and list items. Each returns `None` for input the grammar
//! rejects, which the decoder turns into the field's default or a dropped item.

use super::decode::RawItem;
use super::fields::ItemRule;
use super::limits::{MAX_KEY_BYTES, MAX_NUMBER_CHARS, MAX_SEARCH_BYTES, VIEW_STATE_MAX_ITEMS};
use super::text::{has_control, unescape};
use crate::app::view_model::{SortState, sort_from_token};
use crate::filter::FloatRange;
use crate::model::MapColor;

/// `q`: 1 to `MAX_SEARCH_BYTES` decoded bytes without control characters.
#[must_use]
pub(super) fn search(value: &str) -> Option<String> {
    let text = unescape(value, MAX_SEARCH_BYTES)?;
    if text.is_empty() || has_control(&text) {
        return None;
    }
    Some(text)
}

/// `l`: 1 to `MAX_KEY_BYTES` decoded bytes.
#[must_use]
pub(super) fn key(value: &str) -> Option<String> {
    let text = unescape(value, MAX_KEY_BYTES)?;
    if text.is_empty() {
        return None;
    }
    Some(text)
}

/// A list field. An empty value is the empty list. A non-empty value whose items are all dropped
/// is `None`, so the field takes its default (for Type, land only, not every kind).
#[must_use]
pub(super) fn list(value: &str, rule: ItemRule) -> Option<Vec<RawItem>> {
    if value.is_empty() {
        return Some(Vec::new());
    }
    let mut items = Vec::new();
    for item in value.split('*').take(VIEW_STATE_MAX_ITEMS) {
        if let Some(raw) = list_item(item, rule) {
            items.push(raw);
        }
    }
    if items.is_empty() { None } else { Some(items) }
}

/// One list item under a facet's rule.
fn list_item(item: &str, rule: ItemRule) -> Option<RawItem> {
    if item == "!" {
        return match rule {
            ItemRule::Keys { missing: true } => Some(RawItem::Missing),
            ItemRule::Keys { missing: false } | ItemRule::Words(_) | ItemRule::Tier => None,
        };
    }
    let text = key(item)?;
    let accepted = match rule {
        ItemRule::Words(words) => words.contains(&text.as_str()),
        ItemRule::Keys { .. } => true,
        ItemRule::Tier => is_tier(&text),
    };
    if accepted {
        Some(RawItem::Key(text))
    } else {
        None
    }
}

/// Decimal integer 1 to 255 without leading zeros or sign.
fn is_tier(text: &str) -> bool {
    let canonical = text.bytes().all(|byte| byte.is_ascii_digit()) && !text.starts_with('0');
    canonical && matches!(text.parse::<u8>(), Ok(value) if value >= 1)
}

/// `[-]digits[.digits]` of at most `MAX_NUMBER_CHARS` characters with a finite `f32` value.
/// The grammar is checked by hand because `str::parse` also accepts exponents, `inf`, `nan`, a
/// leading `+` and bare fractions. Negative zero becomes zero.
#[must_use]
pub(super) fn number(token: &str) -> Option<f32> {
    if token.is_empty() || token.len() > MAX_NUMBER_CHARS {
        return None;
    }
    let body = match token.strip_prefix('-') {
        Some(rest) => rest,
        None => token,
    };
    let (integer, fraction) = match body.split_once('.') {
        Some((integer, fraction)) => (integer, Some(fraction)),
        None => (body, None),
    };
    if !all_digits(integer) {
        return None;
    }
    if let Some(fraction) = fraction
        && !all_digits(fraction)
    {
        return None;
    }
    match token.parse::<f32>() {
        // Adding positive zero turns -0.0 into 0.0 and leaves every other value unchanged.
        Ok(value) if value.is_finite() => Some(value + 0.0),
        Ok(_) | Err(_) => None,
    }
}

/// Non-empty ASCII digits.
fn all_digits(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit())
}

/// `min*max` with either side empty; any malformed side rejects the whole range.
#[must_use]
pub(super) fn range(value: &str) -> Option<FloatRange> {
    let (min, max) = value.split_once('*')?;
    if max.contains('*') {
        return None;
    }
    Some(FloatRange {
        min: optional_number(min)?,
        max: optional_number(max)?,
    })
}

/// Empty side is `Some(None)`, a valid number `Some(Some(value))`, anything else `None`.
fn optional_number(token: &str) -> Option<Option<f32>> {
    if token.is_empty() {
        return Some(None);
    }
    number(token).map(Some)
}

/// Exactly six hexadecimal digits in either case.
#[must_use]
pub(super) fn rgb(value: &str) -> Option<MapColor> {
    if value.len() != 6 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    u32::from_str_radix(value, 16).ok().map(MapColor)
}

/// `0` or `1`.
#[must_use]
pub(super) fn flag(value: &str) -> Option<bool> {
    match value {
        "0" => Some(false),
        "1" => Some(true),
        _ => None,
    }
}

/// `column*a` or `column*d`.
#[must_use]
pub(super) fn sort(value: &str) -> Option<SortState> {
    let (column, direction) = value.split_once('*')?;
    let field = sort_from_token(column)?;
    let ascending = match direction {
        "a" => true,
        "d" => false,
        _ => return None,
    };
    Some(SortState { field, ascending })
}
