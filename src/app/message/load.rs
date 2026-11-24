//! Data, language pack and font pack loading results, and the load-error controls.

use std::sync::Arc;

use crate::AppError;
use crate::app::handoff::Handoff;
use crate::app::i18n::{FontPack, Text};
use crate::app::load::LoadedData;
use crate::lang_pack::{GameLanguage, ValidatedLanguagePack};

/// Loading results and load-error controls.
#[derive(Clone, Debug)]
pub(crate) enum LoadMessage {
    /// The browser painted its first frame; decoding may start.
    Painted,
    /// Progress of the load with this generation; stale generations are ignored.
    Progress {
        /// Load generation that reported.
        generation: u64,
        /// Catalog text for the stage.
        stage: Text,
        /// Completed fraction in `0.0..=1.0`.
        fraction: f32,
    },
    /// Final result of the load with this generation.
    Finished {
        /// Load generation that finished.
        generation: u64,
        /// Prepared dataset and engine, or the failure.
        result: Result<Handoff<LoadedData>, Arc<AppError>>,
    },
    /// Desktop installation directory field text on the error card.
    InstallPath(String),
    /// Desktop Retry: load again with the entered directory.
    Retry,
    /// Desktop Rebuild data: import from the game files and replace the external bundles.
    Rebuild,
    /// Browser: compressed language pack bytes, validated inside update.
    #[cfg_attr(
        not(all(feature = "web", target_family = "wasm")),
        allow(dead_code, reason = "only the browser loader fetches pack bytes")
    )]
    LanguagePackFetched {
        /// Requested language.
        language: GameLanguage,
        /// Pack bytes or the fetch failure.
        result: Result<Arc<[u8]>, Arc<AppError>>,
    },
    /// Desktop: pack decoded and validated off the UI thread.
    #[cfg_attr(
        not(feature = "desktop"),
        allow(dead_code, reason = "only the desktop loader validates off the UI thread")
    )]
    LanguagePackValidated {
        /// Requested language.
        language: GameLanguage,
        /// Validated pack or the failure.
        result: Result<Handoff<ValidatedLanguagePack>, Arc<AppError>>,
    },
    /// A font pack was registered with the renderer, or failed to load.
    FontPackLoaded {
        /// Requested pack.
        pack: FontPack,
        /// Success or the failure.
        result: Result<(), Arc<AppError>>,
    },
}
