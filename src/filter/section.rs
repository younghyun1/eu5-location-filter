//! Language sections: per-location search text, trigram postings, and label-ordered sorts.
//!
//! The English section lives in the filter index; every language pack carries its own. Both
//! are built and checked here so a pack and the index cannot disagree about the format.

use super::{index, search, sort, sort_compare::SortKeys, text::fold_search};
use crate::AppError;
use crate::lang_pack::{LABEL_SORT_FIELDS, ResolvedText, StoredLanguageSection};
use crate::model::{Dataset, LocationRecord};

/// Builds a section from resolved display text; `text` is English when built for the index.
pub(super) fn build(dataset: &Dataset, text: &ResolvedText<'_>) -> StoredLanguageSection {
    let searchable: Vec<String> = dataset
        .stored
        .locations
        .iter()
        .enumerate()
        .map(|(slot, record)| search_text(dataset, record, text.names.get(slot).copied()))
        .collect();
    let trigrams = search::build(&searchable);
    let orders = sort::label_orders(dataset, &SortKeys::labels(text));
    StoredLanguageSection {
        searchable,
        trigrams,
        orders,
    }
}

/// Checks dimensions, postings, the order field list, and every permutation.
///
/// With `text`, each search string must also equal the one recomputed from that text, which
/// proves an untrusted pack's search data belongs to its own labels. The index's English
/// section is paired with its dataset by the bundle itself and skips that comparison.
pub(super) fn validate(
    section: &StoredLanguageSection,
    dataset: &Dataset,
    text: Option<&ResolvedText<'_>>,
) -> Result<(), AppError> {
    let count = dataset.stored.locations.len();
    if section.searchable.len() != count {
        return Err(AppError::InvalidData(
            "language section search text does not match the location count".to_owned(),
        ));
    }
    if let Some(text) = text {
        let mismatch = dataset
            .stored
            .locations
            .iter()
            .zip(&section.searchable)
            .enumerate()
            .any(|(slot, (record, stored))| {
                *stored != search_text(dataset, record, text.names.get(slot).copied())
            });
        if mismatch {
            return Err(AppError::InvalidData(
                "language section search text does not match its labels".to_owned(),
            ));
        }
    }
    search::validate(&section.trigrams, &section.searchable)?;
    if section.orders.len() != LABEL_SORT_FIELDS.len()
        || section
            .orders
            .iter()
            .zip(LABEL_SORT_FIELDS)
            .any(|(order, field)| order.field != field)
    {
        return Err(AppError::InvalidData(
            "language section sort fields are missing, duplicated, or out of order".to_owned(),
        ));
    }
    for order in &section.orders {
        index::validate_order(order, count)?;
    }
    Ok(())
}

/// Folded localized name when it adds something, then the folded English name and identifier,
/// joined by NUL so a query cannot match across two of them.
///
/// The English form is exactly `fold_search(name) NUL fold_search(key)`, so English search is
/// unchanged by the split into sections.
fn search_text(dataset: &Dataset, record: &LocationRecord, localized: Option<&str>) -> String {
    let english = fold_search(dataset.symbol(record.name).unwrap_or_default());
    let key = fold_search(dataset.symbol(record.key).unwrap_or_default());
    let localized = localized
        .map(fold_search)
        .filter(|value| !value.is_empty() && *value != english);
    let extra = localized.as_ref().map_or(0, |value| value.len() + 1);
    let mut output = String::with_capacity(extra + english.len() + 1 + key.len());
    if let Some(localized) = &localized {
        output.push_str(localized);
        output.push('\0');
    }
    output.push_str(&english);
    output.push('\0');
    output.push_str(&key);
    output
}
