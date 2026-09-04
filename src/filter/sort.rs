//! Adaptive output ordering from bundled permutations and their inverse ranks.

use super::{SortField, bitmap::Bitmap, index::StoredSortOrder, sort_compare::compare};
use crate::model::{Dataset, LocationId};
use std::collections::HashMap;

pub(super) struct SortOrders {
    values: HashMap<SortField, StoredSortOrder>,
}

impl SortOrders {
    pub(super) fn new(dataset: &Dataset, folded: &[String]) -> Self {
        let base: Vec<_> = dataset
            .stored
            .locations
            .iter()
            .map(|record| record.id)
            .collect();
        let mut values = HashMap::with_capacity(SortField::ALL.len());
        for field in SortField::ALL {
            let mut ascending = base.clone();
            let mut descending = base.clone();
            ascending.sort_unstable_by(|left, right| {
                compare(dataset, folded, *left, *right, field, true)
            });
            descending.sort_unstable_by(|left, right| {
                compare(dataset, folded, *left, *right, field, false)
            });
            values.insert(
                field,
                StoredSortOrder {
                    field,
                    ascending_ranks: ranks(&ascending),
                    descending_ranks: ranks(&descending),
                    ascending,
                    descending,
                },
            );
        }
        Self { values }
    }

    #[cfg(test)]
    pub(super) fn get(&self, field: SortField, ascending: bool) -> Option<&[LocationId]> {
        self.values.get(&field).map(|order| {
            if ascending {
                order.ascending.as_slice()
            } else {
                order.descending.as_slice()
            }
        })
    }

    pub(super) fn select(
        &self,
        mask: &Bitmap,
        field: SortField,
        ascending: bool,
    ) -> Vec<LocationId> {
        let Some(stored) = self.values.get(&field) else {
            return Vec::new();
        };
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

    pub(super) fn into_stored(mut self) -> Vec<StoredSortOrder> {
        SortField::ALL
            .into_iter()
            .filter_map(|field| self.values.remove(&field))
            .collect()
    }

    pub(super) fn from_stored(stored: Vec<StoredSortOrder>) -> Self {
        Self {
            values: stored
                .into_iter()
                .map(|order| (order.field, order))
                .collect(),
        }
    }
}

pub(super) fn use_ranks(count: usize, total: usize) -> bool {
    count.saturating_mul(count.checked_ilog2().unwrap_or(0) as usize + 1) < total / 2
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
