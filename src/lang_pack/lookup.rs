//! Display-text lookup with English fallback, for the table, detail pane, and checklists.

use super::{GameLanguage, PackStringId, StoredLanguagePack, StoredLanguageSection};
use crate::model::{Dataset, LocationId, SymbolId};

/// A pack checked against one dataset; only [`validate_pack`](super::validate_pack) builds it.
#[derive(Clone, Debug)]
pub struct ValidatedLanguagePack {
    /// Validated payload.
    pub(super) stored: StoredLanguagePack,
    /// Dense symbol-to-label table sized by the dataset dictionary, built once at validation so
    /// a table cell lookup is one index instead of a search.
    pub(super) label_by_symbol: Vec<Option<PackStringId>>,
}

impl ValidatedLanguagePack {
    /// Language of the pack.
    #[must_use]
    pub const fn language(&self) -> GameLanguage {
        self.stored.language
    }

    /// Validated payload, for diff and rebuild tooling.
    #[must_use]
    pub const fn stored(&self) -> &StoredLanguagePack {
        &self.stored
    }

    /// Search and sort data the filter engine uses while this pack is active.
    #[must_use]
    pub const fn section(&self) -> &StoredLanguageSection {
        &self.stored.section
    }

    /// Localized name, or `None` when the English name applies.
    #[must_use]
    pub fn name(&self, id: LocationId) -> Option<&str> {
        let slot = usize::try_from(id.0).ok()?;
        localized_name(&self.stored, slot)
    }

    /// Localized label, or `None` when the English label applies.
    #[must_use]
    pub fn label(&self, symbol: SymbolId) -> Option<&str> {
        let slot = usize::try_from(symbol.0).ok()?;
        let text = self.label_by_symbol.get(slot).copied().flatten()?;
        pack_text(&self.stored, text)
    }
}

/// Display text for one language, falling back to English and then to the identifier.
#[derive(Clone, Copy, Debug)]
pub struct Labels<'a> {
    dataset: &'a Dataset,
    pack: Option<&'a ValidatedLanguagePack>,
}

impl<'a> Labels<'a> {
    /// Combines a dataset with the active pack; `None` means English.
    #[must_use]
    pub const fn new(dataset: &'a Dataset, pack: Option<&'a ValidatedLanguagePack>) -> Self {
        Self { dataset, pack }
    }

    /// Language whose text this view prefers.
    #[must_use]
    pub const fn language(&self) -> GameLanguage {
        match self.pack {
            Some(pack) => pack.language(),
            None => GameLanguage::English,
        }
    }

    /// Underlying dataset.
    #[must_use]
    pub const fn dataset(&self) -> &'a Dataset {
        self.dataset
    }

    /// Location name in the preferred language, else English.
    #[must_use]
    pub fn name(&self, id: LocationId) -> Option<&'a str> {
        let dataset = self.dataset;
        if let Some(text) = self.pack.and_then(|pack| pack.name(id)) {
            return Some(text);
        }
        let record = dataset.location(id)?;
        dataset.symbol(record.name)
    }

    /// Symbol label in the preferred language, else English, else the identifier.
    #[must_use]
    pub fn label(&self, symbol: SymbolId) -> Option<&'a str> {
        let dataset = self.dataset;
        if let Some(text) = self.pack.and_then(|pack| pack.label(symbol)) {
            return Some(text);
        }
        dataset.label(symbol)
    }
}

/// Dense display text used to build a [`StoredLanguageSection`] offline.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ResolvedText<'a> {
    /// Display name per location ID.
    pub names: Vec<&'a str>,
    /// Display label per dataset symbol, with the identifier as the last fallback.
    pub labels: Vec<&'a str>,
}

/// Resolves every name and label for a stored pack, or for English when `pack` is `None`.
///
/// Out-of-range pack references fall back to English, so this is safe on unvalidated input;
/// the importer calls it before the pack's own section exists.
#[must_use]
pub fn resolve_text<'a>(
    dataset: &'a Dataset,
    pack: Option<&'a StoredLanguagePack>,
) -> ResolvedText<'a> {
    let mut names = Vec::with_capacity(dataset.stored.locations.len());
    for (slot, record) in dataset.stored.locations.iter().enumerate() {
        let localized = pack.and_then(|pack| localized_name(pack, slot));
        let english = dataset.symbol(record.name);
        names.push(localized.or(english).unwrap_or_default());
    }
    let mut labels = Vec::with_capacity(dataset.stored.dictionary.len());
    for slot in 0..dataset.stored.dictionary.len() {
        let label = match u32::try_from(slot) {
            Ok(raw) => dataset.label(SymbolId(raw)),
            Err(_) => None,
        };
        labels.push(label.unwrap_or_default());
    }
    if let Some(pack) = pack {
        for label in &pack.labels {
            let slot = usize::try_from(label.key.0).ok();
            let target = slot.and_then(|slot| labels.get_mut(slot));
            if let (Some(target), Some(text)) = (target, pack_text(pack, label.value)) {
                *target = text;
            }
        }
    }
    ResolvedText { names, labels }
}

fn localized_name(pack: &StoredLanguagePack, slot: usize) -> Option<&str> {
    let text = pack.location_names.get(slot).copied().flatten()?;
    pack_text(pack, text)
}

fn pack_text(pack: &StoredLanguagePack, id: PackStringId) -> Option<&str> {
    let slot = usize::try_from(id.0).ok()?;
    pack.strings.get(slot).map(String::as_str)
}
