//! Embedded font bytes staged by `build.rs`. A face whose source was missing at build time is
//! an empty file and is skipped, so the build never fails for lack of fonts.

/// IBM Plex Sans Regular, Medium and SemiBold, and Plex Mono Regular and Medium, in the order
/// of `EMBEDDED_FONTS` in `build.rs`.
const EMBEDDED_FACES: [&[u8]; 5] = [
    include_bytes!(concat!(env!("OUT_DIR"), "/fonts/plex-sans-regular.ttf")),
    include_bytes!(concat!(env!("OUT_DIR"), "/fonts/plex-sans-medium.ttf")),
    include_bytes!(concat!(env!("OUT_DIR"), "/fonts/plex-sans-semibold.ttf")),
    include_bytes!(concat!(env!("OUT_DIR"), "/fonts/plex-mono-regular.ttf")),
    include_bytes!(concat!(env!("OUT_DIR"), "/fonts/plex-mono-medium.ttf")),
];

/// Non-empty embedded faces, registered with the application builder at startup.
pub(crate) fn embedded_faces() -> impl Iterator<Item = &'static [u8]> {
    EMBEDDED_FACES.into_iter().filter(|face| !face.is_empty())
}

/// Script packs embedded in the desktop binary, in `FontPack::ALL` order.
#[cfg(feature = "desktop")]
const EMBEDDED_PACKS: [&[u8]; crate::app::i18n::FontPack::COUNT] = [
    include_bytes!(concat!(env!("OUT_DIR"), "/fonts/plex-sans-kr.ttf")),
    include_bytes!(concat!(env!("OUT_DIR"), "/fonts/plex-sans-jp.ttf")),
    include_bytes!(concat!(env!("OUT_DIR"), "/fonts/sans-sc.ttf")),
    include_bytes!(concat!(env!("OUT_DIR"), "/fonts/sans-tc.ttf")),
    include_bytes!(concat!(env!("OUT_DIR"), "/fonts/noto-sans-arabic.ttf")),
    include_bytes!(concat!(env!("OUT_DIR"), "/fonts/noto-sans-devanagari.ttf")),
    include_bytes!(concat!(env!("OUT_DIR"), "/fonts/noto-sans-bengali.ttf")),
    include_bytes!(concat!(env!("OUT_DIR"), "/fonts/noto-sans-telugu.ttf")),
];

/// Embedded bytes of a script pack, or `None` when this build has none.
#[cfg(feature = "desktop")]
#[must_use]
pub(crate) fn embedded_pack(pack: crate::app::i18n::FontPack) -> Option<&'static [u8]> {
    EMBEDDED_PACKS
        .get(pack.index())
        .copied()
        .filter(|bytes| !bytes.is_empty())
}

#[cfg(test)]
mod tests {
    use super::embedded_faces;

    #[test]
    fn embedded_faces_skips_empty_sources() {
        assert!(embedded_faces().all(|face| !face.is_empty()));
    }

    #[cfg(feature = "desktop")]
    #[test]
    fn embedded_packs_are_never_empty_slices() {
        for pack in crate::app::i18n::FontPack::ALL {
            assert!(super::embedded_pack(pack).is_none_or(|bytes| !bytes.is_empty()));
        }
    }
}
