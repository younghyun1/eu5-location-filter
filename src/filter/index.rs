//! Serializable language-independent indexes and the English language section.

use bitcode::{Decode, Encode};

use crate::AppError;
use crate::lang_pack::{LANGUAGE_INDEPENDENT_SORT_FIELDS, StoredLanguageSection, StoredOrder};
use crate::model::{Dataset, EU5_APP_ID, LocationId};

/// Schema version for the independent filter-index bundle.
///
/// Version 6 moved search text, trigram postings and the label-ordered sorts into the English
/// [`StoredLanguageSection`], the shape every language pack shares.
pub const INDEX_FORMAT_VERSION: u16 = 6;

/// Complete payload stored in `eu5-indexes.bitcode.zst`.
///
/// Bitcode encodes fields in declaration order, so the order below is part of the schema.
#[derive(Clone, Debug, Default, Decode, Encode, PartialEq)]
pub struct StoredFilterIndex {
    /// Index schema version.
    pub format_version: u16,
    /// EU5 Steam application ID.
    pub app_id: u32,
    /// Build ID of the paired location dataset.
    pub build_id: u64,
    /// Record count of the paired location dataset.
    pub location_count: u32,
    /// Orders for `LANGUAGE_INDEPENDENT_SORT_FIELDS`, in that order.
    pub orders: Vec<StoredOrder>,
    /// Offline facet and numeric-range indexes.
    pub(super) query: super::query::QueryIndex,
    /// English search text, trigram postings, and label-ordered sorts.
    pub english: StoredLanguageSection,
}

impl StoredFilterIndex {
    /// Validates pairing, dimensions, fields, and every location permutation.
    pub fn validate(&self, dataset: &Dataset) -> Result<(), AppError> {
        if self.format_version != INDEX_FORMAT_VERSION || self.app_id != EU5_APP_ID {
            return Err(AppError::InvalidData(
                "filter index schema or application ID is unsupported".to_owned(),
            ));
        }
        let count = dataset.stored.locations.len();
        if self.build_id != dataset.stored.build_id
            || usize::try_from(self.location_count).ok() != Some(count)
        {
            return Err(AppError::InvalidData(
                "filter index does not match the location dataset".to_owned(),
            ));
        }
        if self.orders.len() != LANGUAGE_INDEPENDENT_SORT_FIELDS.len()
            || self
                .orders
                .iter()
                .zip(LANGUAGE_INDEPENDENT_SORT_FIELDS)
                .any(|(order, field)| order.field != field)
        {
            return Err(AppError::InvalidData(
                "filter index sort fields are missing, duplicated, or out of order".to_owned(),
            ));
        }
        for order in &self.orders {
            validate_order(order, count)?;
        }
        self.query.validate(dataset)?;
        super::section::validate(&self.english, dataset, None)
    }
}

/// Checks that both directions are permutations of `0..count` with matching inverse ranks.
pub(super) fn validate_order(order: &StoredOrder, count: usize) -> Result<(), AppError> {
    validate_permutation(&order.ascending, count)?;
    validate_permutation(&order.descending, count)?;
    validate_ranks(&order.ascending, &order.ascending_ranks)?;
    validate_ranks(&order.descending, &order.descending_ranks)
}

pub(super) fn validate_ranks(order: &[LocationId], ranks: &[u32]) -> Result<(), AppError> {
    if ranks.len() != order.len()
        || order
            .iter()
            .enumerate()
            .any(|(rank, id)| ranks.get(id.0 as usize).copied() != u32::try_from(rank).ok())
    {
        return Err(AppError::InvalidData(
            "sort ranks are not the inverse permutation".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn validate_permutation(values: &[LocationId], count: usize) -> Result<(), AppError> {
    if values.len() != count {
        return Err(AppError::InvalidData(
            "filter index order has an invalid length".to_owned(),
        ));
    }
    let mut seen = vec![false; count];
    for id in values {
        let index = usize::try_from(id.0)
            .map_err(|error| AppError::InvalidData(format!("filter index ID overflow: {error}")))?;
        let Some(slot) = seen.get_mut(index) else {
            return Err(AppError::InvalidData(
                "filter index contains an out-of-range location ID".to_owned(),
            ));
        };
        if *slot {
            return Err(AppError::InvalidData(
                "filter index order contains a duplicate location ID".to_owned(),
            ));
        }
        *slot = true;
    }
    Ok(())
}
