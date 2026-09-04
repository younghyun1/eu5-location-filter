//! Bundled byte-trigram postings; exact verification preserves substring semantics.

use bitcode::{Decode, Encode};
use std::collections::BTreeMap;

use super::{bitmap::Bitmap, posting::validate_ids};
use crate::{AppError, model::LocationId};

#[derive(Clone, Debug, Decode, Encode, PartialEq)]
pub(super) struct Trigram {
    pub(super) key: [u8; 3],
    pub(super) ids: Vec<LocationId>,
}

pub(super) fn build(searchable: &[String]) -> Vec<Trigram> {
    let mut entries = BTreeMap::<_, Vec<_>>::new();
    for (index, text) in searchable.iter().enumerate() {
        let Ok(id) = u32::try_from(index).map(LocationId) else {
            continue;
        };
        for key in trigrams(text) {
            let ids = entries.entry(key).or_default();
            if ids.last() != Some(&id) {
                ids.push(id);
            }
        }
    }
    entries
        .into_iter()
        .map(|(key, ids)| Trigram { key, ids })
        .collect()
}

pub(super) fn apply(
    index: &[Trigram],
    searchable: &[String],
    query: &str,
    mask: &mut Bitmap,
    scratch: &mut Bitmap,
) {
    if query.is_empty() {
        return;
    }
    let mut rarest: Option<&[LocationId]> = None;
    for key in trigrams(query) {
        let Some(posting) = index
            .binary_search_by_key(&key, |entry| entry.key)
            .ok()
            .and_then(|slot| index.get(slot))
        else {
            mask.clear();
            return;
        };
        if rarest.is_none_or(|ids| posting.ids.len() < ids.len()) {
            rarest = Some(&posting.ids);
        }
    }
    scratch.clear();
    let matches = |id: LocationId| {
        searchable
            .get(id.0 as usize)
            .is_some_and(|text| text.contains(query))
    };
    // The rarest gram is a superset of exact matches. Verifying those short strings
    // is cheaper than intersecting every gram and still rejects reordered/repeated grams.
    if let Some(ids) = rarest.filter(|ids| ids.len() < mask.count()) {
        for id in ids {
            if mask.contains(*id) && matches(*id) {
                scratch.insert(*id);
            }
        }
    } else {
        // One/two-byte input, or an already selective facet: verify only eligible IDs.
        for id in mask.ids() {
            if matches(id) {
                scratch.insert(id);
            }
        }
    }
    mask.intersect(scratch);
}

pub(super) fn validate(index: &[Trigram], searchable: &[String]) -> Result<(), AppError> {
    let max_postings: usize = searchable
        .iter()
        .map(|text| text.len().saturating_sub(2))
        .sum();
    if index.len() > max_postings
        || index
            .windows(2)
            .any(|pair| !matches!(pair, [left, right] if left.key < right.key))
    {
        return Err(AppError::InvalidData(
            "trigram keys are excessive, duplicated, or unordered".to_owned(),
        ));
    }
    let mut total = 0;
    for entry in index {
        validate_ids(&entry.ids, searchable.len())?;
        total += entry.ids.len();
        if entry.ids.is_empty() || total > max_postings {
            return Err(AppError::InvalidData(
                "trigram posting count is invalid".to_owned(),
            ));
        }
    }
    Ok(())
}

fn trigrams(text: &str) -> impl Iterator<Item = [u8; 3]> + '_ {
    // Byte windows also work for UTF-8: a matching string necessarily contains
    // every window, while exact verification removes partial-codepoint candidates.
    text.as_bytes()
        .windows(3)
        .filter_map(|window| window.try_into().ok())
}
