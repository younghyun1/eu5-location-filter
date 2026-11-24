//! Adaptive output ordering from bundled permutations and their inverse ranks.
//!
//! The twelve language-independent orders live in the index; the thirteen label-ordered ones
//! live in a language section. Both use [`StoredOrder`] and the same selection strategy.

use super::{
    SortField,
    bitmap::Bitmap,
    sort_compare::{SortKeys, compare},
};
use crate::lang_pack::{LABEL_SORT_FIELDS, LANGUAGE_INDEPENDENT_SORT_FIELDS, StoredOrder};
use crate::model::{Dataset, LocationId};

/// Orders for `LANGUAGE_INDEPENDENT_SORT_FIELDS`, stored in that order so lookup is positional.
pub(super) struct SortOrders {
    orders: Vec<StoredOrder>,
}

impl SortOrders {
    /// Sorts every language-independent column offline.
    pub(super) fn independent(dataset: &Dataset) -> Self {
        let keys = SortKeys::identifiers(dataset);
        Self {
            orders: build_orders(dataset, &keys, &LANGUAGE_INDEPENDENT_SORT_FIELDS),
        }
    }

    /// Order for a language-independent column; `None` for label-ordered columns.
    pub(super) fn get(&self, field: SortField) -> Option<&StoredOrder> {
        find(&LANGUAGE_INDEPENDENT_SORT_FIELDS, &self.orders, field)
    }

    pub(super) fn into_stored(self) -> Vec<StoredOrder> {
        self.orders
    }

    /// Wraps orders that `StoredFilterIndex::validate` already checked field by field.
    pub(super) fn from_stored(orders: Vec<StoredOrder>) -> Self {
        Self { orders }
    }
}

/// Sorts the thirteen label-ordered columns of one language, in `LABEL_SORT_FIELDS` order.
pub(super) fn label_orders(dataset: &Dataset, keys: &SortKeys) -> Vec<StoredOrder> {
    build_orders(dataset, keys, &LABEL_SORT_FIELDS)
}

/// Finds a column's order by its position in `fields`, without searching or allocating.
pub(super) fn find<'a>(
    fields: &[SortField],
    orders: &'a [StoredOrder],
    field: SortField,
) -> Option<&'a StoredOrder> {
    let slot = fields.iter().position(|candidate| *candidate == field)?;
    orders.get(slot).filter(|order| order.field == field)
}

/// Returns the masked IDs in the order's direction.
pub(super) fn select(stored: &StoredOrder, mask: &Bitmap, ascending: bool) -> Vec<LocationId> {
    let (order, ranks) = if ascending {
        (&stored.ascending, &stored.ascending_ranks)
    } else {
        (&stored.descending, &stored.descending_ranks)
    };
    let count = mask.count();
    if count == order.len() {
        return order.clone();
    }
    if count == 0 {
        return Vec::new();
    }
    // Sparse output pays only integer rank comparisons. Dense output scans the
    // permutation once; the conservative crossover avoids sorting broad results.
    if use_ranks(count, order.len()) {
        let mut ids = Vec::with_capacity(count);
        ids.extend(mask.ids());
        ids.sort_unstable_by_key(|id| ranks.get(id.0 as usize).copied().unwrap_or(u32::MAX));
        ids
    } else {
        let mut ids = Vec::with_capacity(count);
        ids.extend(order.iter().copied().filter(|id| mask.contains(*id)));
        ids
    }
}

pub(super) fn use_ranks(count: usize, total: usize) -> bool {
    count.saturating_mul(count.checked_ilog2().unwrap_or(0) as usize + 1) < total / 2
}

fn build_orders(dataset: &Dataset, keys: &SortKeys, fields: &[SortField]) -> Vec<StoredOrder> {
    let base: Vec<_> = dataset
        .stored
        .locations
        .iter()
        .map(|record| record.id)
        .collect();
    fields
        .iter()
        .map(|field| {
            let field = *field;
            let mut ascending = base.clone();
            let mut descending = base.clone();
            ascending
                .sort_unstable_by(|left, right| compare(dataset, keys, *left, *right, field, true));
            descending.sort_unstable_by(|left, right| {
                compare(dataset, keys, *left, *right, field, false)
            });
            StoredOrder {
                field,
                ascending_ranks: ranks(&ascending),
                descending_ranks: ranks(&descending),
                ascending,
                descending,
            }
        })
        .collect()
}

fn ranks(order: &[LocationId]) -> Vec<u32> {
    let mut values = vec![0; order.len()];
    for (rank, id) in order.iter().enumerate() {
        if let Some(slot) = values.get_mut(id.0 as usize) {
            *slot = u32::try_from(rank).unwrap_or(u32::MAX);
        }
    }
    values
}
