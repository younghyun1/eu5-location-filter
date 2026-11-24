//! Language pack decoding and validation shared by both targets, the browser pack path, and
//! font registration. Decoding runs off the UI thread on desktop and inside update in the
//! browser.

use std::borrow::Cow;

use iced::Task;

use crate::AppError;
use crate::app::i18n::FontPack;
use crate::app::message::{LoadMessage, Message};
use crate::lang_pack::{
    GameLanguage, PACK_DIRECTORY, ValidatedLanguagePack, decode_pack, pack_file_name,
    validate_pack,
};
use crate::model::Dataset;

/// Decodes compressed pack bytes and validates them against the dataset for `expected`.
/// Pairing failures are `AppError::PackMismatch`, structural ones `InvalidData`, as classified
/// in the language pack contract.
pub(crate) fn validate_pack_bytes(
    bytes: &[u8],
    dataset: &Dataset,
    expected: GameLanguage,
) -> Result<ValidatedLanguagePack, AppError> {
    let stored = decode_pack(bytes)?;
    validate_pack(stored, dataset, expected)
}

/// Path of the pack for `language` and `build_id` relative to the frame document, or `None`
/// for English, which has no pack. The build ID in the name keeps a cached pack from pairing
/// with a newer dataset.
#[cfg_attr(
    not(all(feature = "web", target_family = "wasm")),
    allow(dead_code, reason = "only the browser loader fetches pack files")
)]
#[must_use]
pub(crate) fn pack_fetch_path(language: GameLanguage, build_id: u64) -> Option<String> {
    pack_file_name(language, build_id).map(|name| format!("{PACK_DIRECTORY}/{name}"))
}

/// The `FontPackLoaded` message for a registration outcome.
pub(crate) fn font_message(pack: FontPack, result: Result<(), AppError>) -> Message {
    Message::Load(LoadMessage::FontPackLoaded {
        pack,
        result: result.map_err(std::sync::Arc::new),
    })
}

/// Registers font bytes with the renderer and answers with `FontPackLoaded`. The renderer keeps
/// the face for the rest of the session; the caller records that so it is never loaded twice.
pub(crate) fn register_font(pack: FontPack, bytes: impl Into<Cow<'static, [u8]>>) -> Task<Message> {
    iced::font::load(bytes).map(move |result| {
        font_message(
            pack,
            result.map_err(|error| {
                AppError::Ui(format!("font pack {} was rejected: {error:?}", pack.file_stem()))
            }),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::{pack_fetch_path, validate_pack_bytes};
    use crate::filter::test_fixture::fixture;
    use crate::lang_pack::GameLanguage;

    #[test]
    fn fetch_path_is_relative_and_carries_the_build() {
        assert_eq!(
            pack_fetch_path(GameLanguage::Korean, 7).as_deref(),
            Some("lang/ko.7.bitcode.zst")
        );
        assert_eq!(pack_fetch_path(GameLanguage::English, 7), None);
    }

    #[test]
    fn garbage_bytes_fail_without_panicking() {
        let dataset = fixture();
        assert!(validate_pack_bytes(&[], &dataset, GameLanguage::French).is_err());
        assert!(validate_pack_bytes(b"not a pack", &dataset, GameLanguage::French).is_err());
    }
}
