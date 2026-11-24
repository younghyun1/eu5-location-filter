//! The 27 site locales with their direction, game language, font and native name.

use super::fonts::FontPack;
use crate::app::layout::Direction;
use crate::app::theme::FONT_SANS_FAMILY;
use crate::lang_pack::GameLanguage;

/// A site locale tag. Variant order is the catalog table order; `build.rs` keeps the same tag
/// list (`LOCALE_TAGS`), so both change together.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum Locale {
    /// `en-US`.
    #[default]
    EnUs,
    /// `ko-KR`.
    KoKr,
    /// `fr-FR`.
    FrFr,
    /// `es-ES`.
    EsEs,
    /// `zh-Hans`.
    ZhHans,
    /// `zh-Hant`.
    ZhHant,
    /// `ja-JP`.
    JaJp,
    /// `de-DE`.
    DeDe,
    /// `ar`.
    Ar,
    /// `hi-IN`.
    HiIn,
    /// `bn-BD`.
    BnBd,
    /// `te-IN`.
    TeIn,
    /// `pt-BR`.
    PtBr,
    /// `id-ID`.
    IdId,
    /// `ru-RU`.
    RuRu,
    /// `ur-PK`.
    UrPk,
    /// `vi-VN`.
    ViVn,
    /// `tl-PH`.
    TlPh,
    /// `fa-IR`.
    FaIr,
    /// `it-IT`.
    ItIt,
    /// `pl-PL`.
    PlPl,
    /// `sv-SE`.
    SvSe,
    /// `sh`.
    Sh,
    /// `ro-RO`.
    RoRo,
    /// `nl-NL`.
    NlNl,
    /// `uk-UA`.
    UkUa,
    /// `tr-TR`.
    TrTr,
}

impl Locale {
    /// Number of supported locales.
    pub(crate) const COUNT: usize = 27;

    /// Every locale in catalog table order.
    pub(crate) const ALL: [Self; Self::COUNT] = [
        Self::EnUs,
        Self::KoKr,
        Self::FrFr,
        Self::EsEs,
        Self::ZhHans,
        Self::ZhHant,
        Self::JaJp,
        Self::DeDe,
        Self::Ar,
        Self::HiIn,
        Self::BnBd,
        Self::TeIn,
        Self::PtBr,
        Self::IdId,
        Self::RuRu,
        Self::UrPk,
        Self::ViVn,
        Self::TlPh,
        Self::FaIr,
        Self::ItIt,
        Self::PlPl,
        Self::SvSe,
        Self::Sh,
        Self::RoRo,
        Self::NlNl,
        Self::UkUa,
        Self::TrTr,
    ];

    /// Position in `ALL` and in the generated catalog table.
    #[must_use]
    pub(crate) const fn index(self) -> usize {
        self as usize
    }

    /// Site tag, exactly as the site writes it.
    #[cfg(any(test, all(feature = "web", target_family = "wasm")))]
    #[must_use]
    pub(crate) const fn tag(self) -> &'static str {
        match self {
            Self::EnUs => "en-US",
            Self::KoKr => "ko-KR",
            Self::FrFr => "fr-FR",
            Self::EsEs => "es-ES",
            Self::ZhHans => "zh-Hans",
            Self::ZhHant => "zh-Hant",
            Self::JaJp => "ja-JP",
            Self::DeDe => "de-DE",
            Self::Ar => "ar",
            Self::HiIn => "hi-IN",
            Self::BnBd => "bn-BD",
            Self::TeIn => "te-IN",
            Self::PtBr => "pt-BR",
            Self::IdId => "id-ID",
            Self::RuRu => "ru-RU",
            Self::UrPk => "ur-PK",
            Self::ViVn => "vi-VN",
            Self::TlPh => "tl-PH",
            Self::FaIr => "fa-IR",
            Self::ItIt => "it-IT",
            Self::PlPl => "pl-PL",
            Self::SvSe => "sv-SE",
            Self::Sh => "sh",
            Self::RoRo => "ro-RO",
            Self::NlNl => "nl-NL",
            Self::UkUa => "uk-UA",
            Self::TrTr => "tr-TR",
        }
    }

    /// Exact, case-sensitive tag lookup; host input that is not one of the 27 tags maps to
    /// `None` and the caller applies `EnUs`.
    #[cfg(all(feature = "web", target_family = "wasm"))]
    #[must_use]
    pub(crate) fn from_tag(tag: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|locale| locale.tag() == tag)
    }

    /// Layout direction.
    #[must_use]
    pub(crate) const fn direction(self) -> Direction {
        match self {
            Self::Ar | Self::FaIr | Self::UrPk => Direction::Rtl,
            _ => Direction::Ltr,
        }
    }

    /// Game language whose names and labels the grid shows; English where the game has no
    /// translation for this locale.
    #[must_use]
    pub(crate) const fn game_language(self) -> GameLanguage {
        match self {
            Self::KoKr => GameLanguage::Korean,
            Self::FrFr => GameLanguage::French,
            Self::EsEs => GameLanguage::Spanish,
            Self::ZhHans => GameLanguage::SimplifiedChinese,
            Self::JaJp => GameLanguage::Japanese,
            Self::DeDe => GameLanguage::German,
            Self::PtBr => GameLanguage::BrazilianPortuguese,
            Self::RuRu => GameLanguage::Russian,
            Self::PlPl => GameLanguage::Polish,
            Self::TrTr => GameLanguage::Turkish,
            _ => GameLanguage::English,
        }
    }

    /// Script font pack this locale needs, or `None` when the embedded Plex subset covers it.
    #[must_use]
    pub(crate) const fn font_pack(self) -> Option<FontPack> {
        match self {
            Self::KoKr => Some(FontPack::PlexSansKr),
            Self::ZhHans => Some(FontPack::SansSc),
            Self::ZhHant => Some(FontPack::SansTc),
            Self::JaJp => Some(FontPack::PlexSansJp),
            Self::Ar => Some(FontPack::NotoSansArabic),
            Self::HiIn => Some(FontPack::NotoSansDevanagari),
            Self::BnBd => Some(FontPack::NotoSansBengali),
            Self::TeIn => Some(FontPack::NotoSansTelugu),
            Self::UrPk => Some(FontPack::NotoSansArabic),
            Self::FaIr => Some(FontPack::NotoSansArabic),
            _ => None,
        }
    }

    /// Sans family for reading text: the script pack family when the locale has one.
    #[must_use]
    pub(crate) const fn font_family(self) -> &'static str {
        match self.font_pack() {
            Some(pack) => pack.family(),
            None => FONT_SANS_FAMILY,
        }
    }

    /// Name of the language written in that language, for the desktop Language menu.
    #[must_use]
    pub(crate) const fn native_name(self) -> &'static str {
        match self {
            Self::EnUs => "English (US)",
            Self::KoKr => "한국어",
            Self::FrFr => "Français",
            Self::EsEs => "Español",
            Self::ZhHans => "简体中文",
            Self::ZhHant => "繁體中文",
            Self::JaJp => "日本語",
            Self::DeDe => "Deutsch",
            Self::Ar => "العربية",
            Self::HiIn => "हिन्दी",
            Self::BnBd => "বাংলা",
            Self::TeIn => "తెలుగు",
            Self::PtBr => "Português (Brasil)",
            Self::IdId => "Bahasa Indonesia",
            Self::RuRu => "Русский",
            Self::UrPk => "اردو",
            Self::ViVn => "Tiếng Việt",
            Self::TlPh => "Filipino",
            Self::FaIr => "فارسی",
            Self::ItIt => "Italiano",
            Self::PlPl => "Polski",
            Self::SvSe => "Svenska",
            Self::Sh => "Srpskohrvatski",
            Self::RoRo => "Română",
            Self::NlNl => "Nederlands",
            Self::UkUa => "Українська",
            Self::TrTr => "Türkçe",
        }
    }
}
