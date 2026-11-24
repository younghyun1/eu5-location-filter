//! Script font packs: lazily fetched in the browser, embedded in the desktop binary.

/// One script font pack. File stems equal the staged and fetched names; family names must
/// equal the family recorded in each font's name table, because iced selects faces by family.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum FontPack {
    /// IBM Plex Sans KR.
    PlexSansKr,
    /// IBM Plex Sans JP.
    PlexSansJp,
    /// Simplified Chinese face subset to catalog and name-pack glyphs.
    SansSc,
    /// Traditional Chinese face subset to catalog glyphs.
    SansTc,
    /// Noto Sans Arabic, shipped whole so shaping tables survive.
    NotoSansArabic,
    /// Noto Sans Devanagari, shipped whole.
    NotoSansDevanagari,
    /// Noto Sans Bengali, shipped whole.
    NotoSansBengali,
    /// Noto Sans Telugu, shipped whole.
    NotoSansTelugu,
}

impl FontPack {
    /// Number of packs.
    pub(crate) const COUNT: usize = 8;

    /// Every pack, in `index` order.
    #[cfg(test)]
    pub(crate) const ALL: [Self; Self::COUNT] = [
        Self::PlexSansKr,
        Self::PlexSansJp,
        Self::SansSc,
        Self::SansTc,
        Self::NotoSansArabic,
        Self::NotoSansDevanagari,
        Self::NotoSansBengali,
        Self::NotoSansTelugu,
    ];

    /// Position in `ALL`, used for fixed-size bookkeeping arrays.
    #[must_use]
    pub(crate) const fn index(self) -> usize {
        self as usize
    }

    /// File stem under `assets/fonts/`, `OUT_DIR/fonts/` and the served `fonts/` directory.
    #[must_use]
    pub(crate) const fn file_stem(self) -> &'static str {
        match self {
            Self::PlexSansKr => "plex-sans-kr",
            Self::PlexSansJp => "plex-sans-jp",
            Self::SansSc => "sans-sc",
            Self::SansTc => "sans-tc",
            Self::NotoSansArabic => "noto-sans-arabic",
            Self::NotoSansDevanagari => "noto-sans-devanagari",
            Self::NotoSansBengali => "noto-sans-bengali",
            Self::NotoSansTelugu => "noto-sans-telugu",
        }
    }

    /// Path relative to the frame document, as passed to the host fetch.
    #[cfg(all(feature = "web", target_family = "wasm"))]
    #[must_use]
    pub(crate) const fn fetch_path(self) -> &'static str {
        match self {
            Self::PlexSansKr => "fonts/plex-sans-kr.ttf",
            Self::PlexSansJp => "fonts/plex-sans-jp.ttf",
            Self::SansSc => "fonts/sans-sc.ttf",
            Self::SansTc => "fonts/sans-tc.ttf",
            Self::NotoSansArabic => "fonts/noto-sans-arabic.ttf",
            Self::NotoSansDevanagari => "fonts/noto-sans-devanagari.ttf",
            Self::NotoSansBengali => "fonts/noto-sans-bengali.ttf",
            Self::NotoSansTelugu => "fonts/noto-sans-telugu.ttf",
        }
    }

    /// Font family name inside the pack.
    #[must_use]
    pub(crate) const fn family(self) -> &'static str {
        match self {
            Self::PlexSansKr => "IBM Plex Sans KR",
            Self::PlexSansJp => "IBM Plex Sans JP",
            Self::SansSc => "IBM Plex Sans SC",
            Self::SansTc => "IBM Plex Sans TC",
            Self::NotoSansArabic => "Noto Sans Arabic",
            Self::NotoSansDevanagari => "Noto Sans Devanagari",
            Self::NotoSansBengali => "Noto Sans Bengali",
            Self::NotoSansTelugu => "Noto Sans Telugu",
        }
    }
}
