//! Fixed-size query masks and compact iteration over matching IDs.

use crate::model::LocationId;

pub(super) struct Bitmap {
    pub(super) words: Vec<u64>,
}

impl Bitmap {
    pub(super) fn empty(count: usize) -> Self {
        Self {
            words: vec![0; count.div_ceil(64)],
        }
    }

    pub(super) fn full(count: usize) -> Self {
        let mut value = Self {
            words: vec![u64::MAX; count.div_ceil(64)],
        };
        if !count.is_multiple_of(64)
            && let Some(last) = value.words.last_mut()
        {
            *last = (1_u64 << (count % 64)) - 1;
        }
        value
    }

    pub(super) fn insert(&mut self, id: LocationId) {
        if let Some(word) = self.words.get_mut(id.0 as usize / 64) {
            *word |= 1_u64 << (id.0 % 64);
        }
    }

    pub(super) fn contains(&self, id: LocationId) -> bool {
        self.words
            .get(id.0 as usize / 64)
            .is_some_and(|word| word & (1_u64 << (id.0 % 64)) != 0)
    }

    pub(super) fn intersect(&mut self, other: &Self) {
        for (word, other) in self.words.iter_mut().zip(&other.words) {
            *word &= other;
        }
    }

    pub(super) fn subtract(&mut self, other: &Self) {
        for (word, other) in self.words.iter_mut().zip(&other.words) {
            *word &= !other;
        }
    }

    pub(super) fn clear(&mut self) {
        self.words.fill(0);
    }

    pub(super) fn count(&self) -> usize {
        self.words
            .iter()
            .map(|word| word.count_ones() as usize)
            .sum()
    }

    pub(super) fn ids(&self) -> impl Iterator<Item = LocationId> + '_ {
        ids(&self.words)
    }
}

pub(super) fn ids(words: &[u64]) -> impl Iterator<Item = LocationId> + '_ {
    words.iter().enumerate().flat_map(|(index, word)| {
        let mut remaining = *word;
        std::iter::from_fn(move || {
            if remaining == 0 {
                return None;
            }
            let bit = remaining.trailing_zeros() as usize;
            remaining &= remaining - 1;
            u32::try_from(index * 64 + bit).ok().map(LocationId)
        })
    })
}
