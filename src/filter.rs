//! Allocation-bounded queries over bundled facet, numeric, search, and sort indexes.
//!
//! Language-independent data (facets, numeric ranges, and twelve sort orders) is stored once in
//! the index. Search text, trigram postings, and the thirteen label-ordered sorts form a
//! language section: the English one lives in the index, every other language in its pack.
//! Installing a pack switches which section queries read, without touching the rest.

use std::sync::Arc;

use crate::AppError;
use crate::lang_pack::{
    self, GameLanguage, LABEL_SORT_FIELDS, Labels, ResolvedText, StoredLanguagePack,
    StoredLanguageSection, StoredOrder, ValidatedLanguagePack,
};
use crate::model::{Dataset, LocationId};

mod bitmap;
mod facets;
mod index;
#[cfg(test)]
mod indexed_tests;
mod numeric;
#[cfg(test)]
mod pack_tests;
mod posting;
mod query;
#[cfg(test)]
mod reference;
mod search;
mod section;
mod sort;
mod sort_compare;
#[cfg(test)]
pub(crate) mod test_fixture;
#[cfg(test)]
mod tests;
mod text;
#[cfg(all(test, feature = "desktop"))]
mod timings;
mod types;

pub use index::StoredFilterIndex;
use sort::SortOrders;
pub(crate) use text::fold_search;
pub use types::{FilterSet, FloatRange, SortField, parse_optional_number};

/// Precomputed indexes plus at most one installed language pack; queries allocate only
/// bounded masks and result IDs.
pub struct FilterEngine {
    dataset: Arc<Dataset>,
    /// `lang_pack::dictionary_digest` of the dataset, computed once at restore so pack
    /// validation and installation can pair without rehashing the dictionary.
    dictionary_digest: u64,
    sort_orders: SortOrders,
    query_index: query::QueryIndex,
    english: StoredLanguageSection,
    /// Active non-English pack; one at a time, so memory stays bounded by one pack.
    pack: Option<ValidatedLanguagePack>,
    #[cfg(test)]
    reference_food: std::collections::HashSet<crate::model::SymbolId>,
}

impl FilterEngine {
    /// Builds indexes for an external dataset or a synthetic test fixture.
    #[must_use]
    pub fn new(dataset: Arc<Dataset>) -> Self {
        let stored = Self::build_stored_index(&dataset);
        Self::restore(dataset, stored)
    }

    /// Builds the deterministic payload offline, without accessing the game installation.
    #[must_use]
    pub fn build_stored_index(dataset: &Dataset) -> StoredFilterIndex {
        let text = lang_pack::resolve_text(dataset, None);
        StoredFilterIndex {
            format_version: index::INDEX_FORMAT_VERSION,
            app_id: crate::model::EU5_APP_ID,
            build_id: dataset.stored.build_id,
            location_count: u32::try_from(dataset.stored.locations.len()).unwrap_or(u32::MAX),
            orders: SortOrders::independent(dataset).into_stored(),
            query: query::QueryIndex::build(dataset),
            english: build_language_section(dataset, &text),
        }
    }

    /// Restores validated indexes without runtime index generation or sorting.
    pub fn from_stored_index(
        dataset: Arc<Dataset>,
        stored: StoredFilterIndex,
    ) -> Result<Self, AppError> {
        stored.validate(&dataset)?;
        Ok(Self::restore(dataset, stored))
    }

    fn restore(dataset: Arc<Dataset>, stored: StoredFilterIndex) -> Self {
        let StoredFilterIndex {
            orders,
            query,
            english,
            ..
        } = stored;
        Self {
            #[cfg(test)]
            reference_food: reference::food_raw_materials(&dataset),
            dictionary_digest: lang_pack::dictionary_digest(&dataset.stored.dictionary),
            dataset,
            sort_orders: SortOrders::from_stored(orders),
            query_index: query,
            english,
            pack: None,
        }
    }

    /// Dataset the engine queries.
    #[must_use]
    pub const fn dataset(&self) -> &Arc<Dataset> {
        &self.dataset
    }

    /// Language of the active section: the installed pack's, else English.
    #[must_use]
    pub fn active_language(&self) -> GameLanguage {
        match &self.pack {
            Some(pack) => pack.language(),
            None => GameLanguage::English,
        }
    }

    /// Display text for the active language with English fallback.
    #[must_use]
    pub fn labels(&self) -> Labels<'_> {
        Labels::new(self.dataset.as_ref(), self.pack.as_ref())
    }

    /// Validates a decoded pack against this engine's dataset, reusing the cached digest.
    pub fn validate_language_pack(
        &self,
        stored: StoredLanguagePack,
        expected: GameLanguage,
    ) -> Result<ValidatedLanguagePack, AppError> {
        lang_pack::validate_paired(stored, &self.dataset, expected, self.dictionary_digest)
    }

    /// Replaces any installed pack after a constant-time pairing re-check.
    ///
    /// A pack validated against another dataset fails with `AppError::PackMismatch` and
    /// leaves the current pack in place.
    pub fn install_language_pack(&mut self, pack: ValidatedLanguagePack) -> Result<(), AppError> {
        let stored = pack.stored();
        let dataset = &self.dataset.stored;
        if stored.build_id != dataset.build_id
            || usize::try_from(stored.location_count).ok() != Some(dataset.locations.len())
            || usize::try_from(stored.symbol_count).ok() != Some(dataset.dictionary.len())
            || stored.dictionary_digest != self.dictionary_digest
        {
            return Err(AppError::PackMismatch(format!(
                "the {} pack was validated against another dataset",
                pack.language().code()
            )));
        }
        self.pack = Some(pack);
        Ok(())
    }

    /// Reverts to English and returns the language that was installed.
    pub fn drop_language_pack(&mut self) -> Option<GameLanguage> {
        self.pack.take().map(|pack| pack.language())
    }

    /// Intersects indexed criteria, verifies search candidates, and orders matching IDs.
    #[must_use]
    pub fn apply(&self, filters: &FilterSet, sort: SortField, ascending: bool) -> Vec<LocationId> {
        let mask = self
            .query_index
            .apply(&self.dataset, self.section(), filters);
        match self.order(sort) {
            Some(order) => sort::select(order, &mask, ascending),
            None => Vec::new(),
        }
    }

    /// Preserves selection only while its ID remains in the filtered index.
    #[must_use]
    pub fn preserve_selection(
        selected: Option<LocationId>,
        visible: &[LocationId],
    ) -> Option<LocationId> {
        selected.filter(|id| visible.contains(id))
    }

    /// Section that search and label-ordered sorts read: the pack's, else English.
    fn section(&self) -> &StoredLanguageSection {
        match &self.pack {
            Some(pack) => pack.section(),
            None => &self.english,
        }
    }

    /// Order for a column, from the active section when the column sorts by a label.
    fn order(&self, field: SortField) -> Option<&StoredOrder> {
        if lang_pack::is_label_dependent(field) {
            sort::find(&LABEL_SORT_FIELDS, &self.section().orders, field)
        } else {
            self.sort_orders.get(field)
        }
    }
}

/// Builds a language section from resolved text, for the index and for pack building.
pub(crate) fn build_language_section(
    dataset: &Dataset,
    text: &ResolvedText<'_>,
) -> StoredLanguageSection {
    section::build(dataset, text)
}

/// Validates a language section; with `text`, its search strings must match that text.
pub(crate) fn validate_language_section(
    section: &StoredLanguageSection,
    dataset: &Dataset,
    text: Option<&ResolvedText<'_>>,
) -> Result<(), AppError> {
    section::validate(section, dataset, text)
}
