//! The eleven game languages and their install, header, and file-name names.

use bitcode::{Decode, Encode};

use crate::AppError;

/// The eleven languages shipped by the EU5 install.
///
/// The variant order is part of the pack schema because bitcode stores the variant index;
/// append new variants only at the end.
#[derive(Clone, Copy, Debug, Default, Decode, Encode, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GameLanguage {
    /// `english`, embedded in the main bundles.
    #[default]
    English,
    /// `french`.
    French,
    /// `german`.
    German,
    /// `spanish`.
    Spanish,
    /// `braz_por`.
    BrazilianPortuguese,
    /// `polish`.
    Polish,
    /// `russian`.
    Russian,
    /// `turkish`.
    Turkish,
    /// `japanese`.
    Japanese,
    /// `korean`.
    Korean,
    /// `simp_chinese`.
    SimplifiedChinese,
}

impl GameLanguage {
    /// Every language in schema order.
    pub const ALL: [Self; 11] = [
        Self::English,
        Self::French,
        Self::German,
        Self::Spanish,
        Self::BrazilianPortuguese,
        Self::Polish,
        Self::Russian,
        Self::Turkish,
        Self::Japanese,
        Self::Korean,
        Self::SimplifiedChinese,
    ];

    /// Languages shipped as separate pack files, in schema order.
    pub const PACKED: [Self; 10] = [
        Self::French,
        Self::German,
        Self::Spanish,
        Self::BrazilianPortuguese,
        Self::Polish,
        Self::Russian,
        Self::Turkish,
        Self::Japanese,
        Self::Korean,
        Self::SimplifiedChinese,
    ];

    /// Directory name, header key, and file-name code, kept in one table so they cannot drift.
    const fn names(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::English => ("english", "l_english", "en"),
            Self::French => ("french", "l_french", "fr"),
            Self::German => ("german", "l_german", "de"),
            Self::Spanish => ("spanish", "l_spanish", "es"),
            Self::BrazilianPortuguese => ("braz_por", "l_braz_por", "pt-br"),
            Self::Polish => ("polish", "l_polish", "pl"),
            Self::Russian => ("russian", "l_russian", "ru"),
            Self::Turkish => ("turkish", "l_turkish", "tr"),
            Self::Japanese => ("japanese", "l_japanese", "ja"),
            Self::Korean => ("korean", "l_korean", "ko"),
            Self::SimplifiedChinese => ("simp_chinese", "l_simp_chinese", "zh-hans"),
        }
    }

    /// Directory under `game/<scope>/localization/` in the install.
    #[must_use]
    pub const fn directory(self) -> &'static str {
        self.names().0
    }

    /// Header key on the first line of every source file, without the trailing colon.
    #[must_use]
    pub const fn header(self) -> &'static str {
        self.names().1
    }

    /// Stable lowercase code used in pack file names, the command line, and host messages.
    #[must_use]
    pub const fn code(self) -> &'static str {
        self.names().2
    }

    /// Whether this language lives in the main bundles instead of a pack file.
    #[must_use]
    pub const fn is_embedded(self) -> bool {
        matches!(self, Self::English)
    }

    /// Looks up a language by its exact [`Self::code`].
    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|language| language.code() == code)
    }
}

impl std::str::FromStr for GameLanguage {
    type Err = AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match Self::from_code(value.trim()) {
            Some(language) => Ok(language),
            None => Err(AppError::InvalidData(format!(
                "unknown game language code: {value}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GameLanguage;

    #[test]
    fn codes_round_trip_and_packed_excludes_english() {
        for language in GameLanguage::ALL {
            let parsed = GameLanguage::from_code(language.code());
            assert_eq!(parsed, Some(language));
            let header = format!("l_{}", language.directory());
            assert_eq!(language.header(), header);
        }
        assert!(!GameLanguage::PACKED.contains(&GameLanguage::English));
        assert!("xx".parse::<GameLanguage>().is_err());
    }
}
