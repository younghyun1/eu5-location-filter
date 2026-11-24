//! Language and font pack bookkeeping. Every collection here has a fixed size, so repeated
//! locale messages cannot grow memory.

use crate::app::i18n::FontPack;
use crate::lang_pack::GameLanguage;

/// Why a pack failed, which decides whether it is fetched again on the next locale change.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PackFailure {
    /// Network or I/O failure; a later selection tries again.
    Retryable,
    /// Corrupt, oversized or not embedded; never retried in this session.
    Permanent,
    /// Built for another dataset; never retried in this session.
    Mismatch,
}

/// Pack state for the session.
#[derive(Clone, Debug, Default)]
pub(crate) struct PackState {
    /// Language whose pack is loading, so a second request for it is not started.
    pub(crate) pending_language: Option<GameLanguage>,
    /// Last failure per language, indexed by position in `GameLanguage::ALL`.
    pub(crate) language_failures: [Option<PackFailure>; 11],
    /// Font packs already registered with the renderer, indexed by `FontPack::index`; fonts
    /// cannot be unloaded, so each loads at most once.
    pub(crate) fonts_loaded: [bool; FontPack::COUNT],
    /// Font packs requested and not yet answered, by the same index.
    pub(crate) fonts_pending: [bool; FontPack::COUNT],
}

impl PackState {
    /// Last recorded failure for a language.
    #[must_use]
    pub(crate) fn language_failure(&self, language: GameLanguage) -> Option<PackFailure> {
        GameLanguage::ALL
            .iter()
            .position(|candidate| *candidate == language)
            .and_then(|index| self.language_failures.get(index).copied().flatten())
    }

    /// Records or clears the failure for a language.
    pub(crate) fn set_language_failure(
        &mut self,
        language: GameLanguage,
        failure: Option<PackFailure>,
    ) {
        let index = GameLanguage::ALL
            .iter()
            .position(|candidate| *candidate == language);
        if let Some(slot) = index.and_then(|index| self.language_failures.get_mut(index)) {
            *slot = failure;
        }
    }
}
