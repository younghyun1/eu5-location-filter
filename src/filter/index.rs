//! Serializable search text and sort orders for the embedded dataset.

use std::collections::HashSet;

use bitcode::{Decode, Encode};

use super::SortField;
use crate::AppError;
use crate::model::{Dataset, EU5_APP_ID, LocationId};

/// Schema version for the independent filter-index bundle.
pub const INDEX_FORMAT_VERSION: u16 = 5;

/// One field's precomputed ascending and descending location order.
#[derive(Clone, Debug, Decode, Encode, PartialEq)]
pub struct StoredSortOrder {
    /// Sort field represented by both vectors.
    pub field: SortField,
    /// Ascending IDs with nulls last.
    pub ascending: Vec<LocationId>,
    /// Descending IDs with nulls last.
    pub descending: Vec<LocationId>,
    /// Inverse ascending permutation, indexed by location ID.
    pub ascending_ranks: Vec<u32>,
    /// Inverse descending permutation, retaining null placement and tie order.
    pub descending_ranks: Vec<u32>,
}

/// Complete payload stored in `eu5-indexes.bitcode.zst`.
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
    /// Compact folded English name and internal ID for each location.
    pub searchable: Vec<String>,
    /// Fixed sort orders in `SortField::ALL` order.
    pub orders: Vec<StoredSortOrder>,
    /// Offline facet, numeric-range, and substring candidate indexes.
    pub(super) query: super::query::QueryIndex,
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
            || self.searchable.len() != count
        {
            return Err(AppError::InvalidData(
                "filter index does not match the location dataset".to_owned(),
            ));
        }
        let mut fields = HashSet::with_capacity(self.orders.len());
        for order in &self.orders {
            if !fields.insert(order.field) {
                return Err(AppError::InvalidData(
                    "filter index repeats a sort field".to_owned(),
                ));
            }
            validate_permutation(&order.ascending, count)?;
            validate_permutation(&order.descending, count)?;
            validate_ranks(&order.ascending, &order.ascending_ranks)?;
            validate_ranks(&order.descending, &order.descending_ranks)?;
        }
        if fields.len() != SortField::ALL.len()
            || SortField::ALL.iter().any(|field| !fields.contains(field))
        {
            return Err(AppError::InvalidData(
                "filter index does not cover every sort field".to_owned(),
            ));
        }
        self.query.validate(dataset, &self.searchable)
    }
}

fn validate_ranks(order: &[LocationId], ranks: &[u32]) -> Result<(), AppError> {
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

fn validate_permutation(values: &[LocationId], count: usize) -> Result<(), AppError> {
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
