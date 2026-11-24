//! The table of output fonts: names, source files and where their codepoints come from.

use crate::lang_pack::GameLanguage;

/// How an output is produced.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Plan {
    /// An embedded IBM Plex face, copied unmodified so it may keep its reserved family name. A
    /// fixed Latin, Cyrillic and punctuation set is required of it; text it lacks is carried by
    /// the Latin fallback face.
    Embedded {
        /// Whether catalog and pack text of the embedded-font locales is required as well.
        with_text: bool,
    },
    /// An embedded face from a wider Latin family, reduced to the characters of the
    /// embedded-font locales that IBM Plex Sans lacks (rare transliteration letters in location
    /// names), so those names never show a missing glyph.
    LatinFallback,
    /// A CJK pack reduced to the codepoints of its locale's catalog and language pack.
    Subset,
    /// A complex-script pack copied unchanged, because shaping tables must survive.
    Whole,
}

impl Plan {
    /// Whether the output is compiled into the binary instead of served as a lazy pack.
    pub(super) const fn is_embedded(self) -> bool {
        matches!(self, Self::Embedded { .. } | Self::LatinFallback)
    }
}

/// Source whose coverage decides what the Latin fallback face must carry.
pub(super) const PRIMARY_SANS_SOURCE: &str = "IBMPlexSans-Regular.ttf";

/// One output font.
#[derive(Clone, Copy, Debug)]
pub(super) struct OutputSpec {
    /// Output file name; must match the names in the host protocol and `build.rs`.
    pub(super) name: &'static str,
    /// Source file name inside the source directory.
    pub(super) source: &'static str,
    /// Production plan.
    pub(super) plan: Plan,
    /// Catalog locale tags (file stems under the catalog directory) whose text this output needs.
    pub(super) locales: &'static [&'static str],
    /// Game languages whose pack text this output needs.
    pub(super) languages: &'static [GameLanguage],
    /// Family name written into a subset. IBM Plex is licensed with the reserved font name
    /// "Plex", which a modified font may not carry, so every subset is renamed; unmodified
    /// copies keep their names.
    pub(super) rename: Option<&'static str>,
}

/// Locales that render with the embedded Plex faces.
pub(super) const EMBEDDED_LOCALES: &[&str] = &[
    "en-US", "fr-FR", "de-DE", "es-ES", "pt-BR", "pl-PL", "tr-TR", "ru-RU", "it-IT", "sv-SE",
    "nl-NL", "ro-RO", "id-ID", "tl-PH", "sh", "vi-VN", "uk-UA",
];

/// Game languages shown with the embedded Plex faces.
pub(super) const EMBEDDED_LANGUAGES: &[GameLanguage] = &[
    GameLanguage::French,
    GameLanguage::German,
    GameLanguage::Spanish,
    GameLanguage::BrazilianPortuguese,
    GameLanguage::Polish,
    GameLanguage::Russian,
    GameLanguage::Turkish,
];

const fn embedded(name: &'static str, source: &'static str, with_text: bool) -> OutputSpec {
    OutputSpec {
        name,
        source,
        plan: Plan::Embedded { with_text },
        locales: if with_text { EMBEDDED_LOCALES } else { &[] },
        languages: if with_text { EMBEDDED_LANGUAGES } else { &[] },
        rename: None,
    }
}

const fn pack(
    name: &'static str,
    source: &'static str,
    plan: Plan,
    locales: &'static [&'static str],
    languages: &'static [GameLanguage],
    rename: Option<&'static str>,
) -> OutputSpec {
    OutputSpec {
        name,
        source,
        plan,
        locales,
        languages,
        rename,
    }
}

/// Every output, in generation order.
pub(super) const OUTPUTS: [OutputSpec; 14] = [
    embedded("plex-sans-regular.ttf", "IBMPlexSans-Regular.ttf", true),
    embedded("plex-sans-medium.ttf", "IBMPlexSans-Medium.ttf", true),
    embedded("plex-sans-semibold.ttf", "IBMPlexSans-SemiBold.ttf", true),
    embedded("plex-mono-regular.ttf", "IBMPlexMono-Regular.ttf", false),
    embedded("plex-mono-medium.ttf", "IBMPlexMono-Medium.ttf", false),
    OutputSpec {
        name: "latin-fallback.ttf",
        source: "NotoSans-Regular.ttf",
        plan: Plan::LatinFallback,
        locales: EMBEDDED_LOCALES,
        languages: EMBEDDED_LANGUAGES,
        rename: Some("EU5 Sans Fallback"),
    },
    pack(
        "plex-sans-kr.ttf",
        "IBMPlexSansKR-Regular.ttf",
        Plan::Subset,
        &["ko-KR"],
        &[GameLanguage::Korean],
        Some("EU5 Sans KR"),
    ),
    pack(
        "plex-sans-jp.ttf",
        "IBMPlexSansJP-Regular.ttf",
        Plan::Subset,
        &["ja-JP"],
        &[GameLanguage::Japanese],
        Some("EU5 Sans JP"),
    ),
    pack(
        "sans-sc.ttf",
        "IBMPlexSansSC-Regular.ttf",
        Plan::Subset,
        &["zh-Hans"],
        &[GameLanguage::SimplifiedChinese],
        Some("EU5 Sans SC"),
    ),
    // Traditional Chinese has no game-language pack; its game text stays English.
    pack(
        "sans-tc.ttf",
        "IBMPlexSansTC-Regular.ttf",
        Plan::Subset,
        &["zh-Hant"],
        &[],
        Some("EU5 Sans TC"),
    ),
    pack(
        "noto-sans-arabic.ttf",
        "NotoSansArabic-Regular.ttf",
        Plan::Whole,
        &["ar", "fa-IR", "ur-PK"],
        &[],
        None,
    ),
    pack(
        "noto-sans-devanagari.ttf",
        "NotoSansDevanagari-Regular.ttf",
        Plan::Whole,
        &["hi-IN"],
        &[],
        None,
    ),
    pack(
        "noto-sans-bengali.ttf",
        "NotoSansBengali-Regular.ttf",
        Plan::Whole,
        &["bn-BD"],
        &[],
        None,
    ),
    pack(
        "noto-sans-telugu.ttf",
        "NotoSansTelugu-Regular.ttf",
        Plan::Whole,
        &["te-IN"],
        &[],
        None,
    ),
];

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::OUTPUTS;

    #[test]
    fn names_are_unique_and_valid_asset_paths() {
        let mut seen = BTreeSet::new();
        for spec in OUTPUTS {
            assert!(seen.insert(spec.name), "duplicate output {}", spec.name);
            let stem = spec.name.strip_suffix(".ttf");
            assert!(stem.is_some_and(|stem| {
                !stem.is_empty()
                    && stem.len() <= 48
                    && stem
                        .bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            }));
        }
    }

    #[test]
    fn script_pack_names_match_the_host_protocol() {
        let packs: Vec<&str> = OUTPUTS
            .iter()
            .filter(|spec| !spec.plan.is_embedded())
            .map(|spec| spec.name)
            .collect();
        assert_eq!(
            packs,
            [
                "plex-sans-kr.ttf",
                "plex-sans-jp.ttf",
                "sans-sc.ttf",
                "sans-tc.ttf",
                "noto-sans-arabic.ttf",
                "noto-sans-devanagari.ttf",
                "noto-sans-bengali.ttf",
                "noto-sans-telugu.ttf",
            ]
        );
    }

    #[test]
    fn embedded_names_match_the_build_script() {
        let embedded: Vec<&str> = OUTPUTS
            .iter()
            .filter(|spec| spec.plan.is_embedded())
            .map(|spec| spec.name)
            .collect();
        assert_eq!(
            embedded,
            [
                "plex-sans-regular.ttf",
                "plex-sans-medium.ttf",
                "plex-sans-semibold.ttf",
                "plex-mono-regular.ttf",
                "plex-mono-medium.ttf",
                "latin-fallback.ttf",
            ]
        );
    }
}
