//! Sparse facets stay smaller than a dense bitmap; common facets use wordwise OR.

use bitcode::{Decode, Encode};

use super::bitmap::Bitmap;
use crate::{AppError, model::LocationId};

#[derive(Clone, Debug, Decode, Encode, PartialEq)]
pub(super) enum Posting {
    Sparse(Vec<LocationId>),
    Dense(Vec<u64>),
}

impl Posting {
    pub(super) fn count(&self) -> usize {
        match self {
            Self::Sparse(ids) => ids.len(),
            Self::Dense(words) => words.iter().map(|word| word.count_ones() as usize).sum(),
        }
    }

    pub(super) fn all(&self, predicate: impl FnMut(LocationId) -> bool) -> bool {
        match self {
            Self::Sparse(ids) => ids.iter().copied().all(predicate),
            Self::Dense(words) => super::bitmap::ids(words).all(predicate),
        }
    }

    pub(super) fn new(ids: Vec<LocationId>, count: usize) -> Self {
        if ids.len() * size_of::<LocationId>() <= count.div_ceil(64) * size_of::<u64>() {
            Self::Sparse(ids)
        } else {
            let mut bitmap = Bitmap::empty(count);
            for id in ids {
                bitmap.insert(id);
            }
            Self::Dense(bitmap.words)
        }
    }

    pub(super) fn union_into(&self, mask: &mut Bitmap) {
        match self {
            Self::Sparse(ids) => {
                for id in ids {
                    mask.insert(*id);
                }
            }
            Self::Dense(words) => {
                for (target, source) in mask.words.iter_mut().zip(words) {
                    *target |= source;
                }
            }
        }
    }

    pub(super) fn validate(&self, count: usize) -> Result<(), AppError> {
        match self {
            Self::Sparse(ids) => validate_ids(ids, count),
            Self::Dense(words) => {
                if words.len() != count.div_ceil(64)
                    || (!count.is_multiple_of(64)
                        && words.last().is_some_and(|last| *last >> (count % 64) != 0))
                {
                    return Err(AppError::InvalidData(
                        "facet bitmap dimensions or padding are invalid".to_owned(),
                    ));
                }
                Ok(())
            }
        }
    }
}

pub(super) fn validate_ids(ids: &[LocationId], count: usize) -> Result<(), AppError> {
    if ids.len() > count
        || ids.iter().any(|id| id.0 as usize >= count)
        || ids
            .windows(2)
            .any(|pair| !matches!(pair, [left, right] if left < right))
    {
        return Err(AppError::InvalidData(
            "posting IDs must be unique, increasing, and in range".to_owned(),
        ));
    }
    Ok(())
}
