//! Font subsetting, isolated in one function.

use std::collections::BTreeSet;

use crate::AppError;

/// Reduces `source` to the glyphs that `keep` maps and returns the new font file.
///
/// The result must keep the family and style names, vertical metrics and the `cmap` entries for
/// every codepoint of `keep` that the source maps. Codepoints of `keep` that the source lacks
/// are skipped silently; the caller verifies the required ones afterwards. Glyph outlines,
/// hinting and the layout tables that the kept glyphs need (`GSUB`, `GPOS`, `GDEF`) must
/// survive, because Hangul, kana and Han text is shaped with them. A variable source must be
/// instanced to its default regular weight first.
///
/// No crate that can subset a font while preserving layout tables is available in this
/// workspace's offline dependency set (it has `ttf-parser`, `read-fonts`, `skrifa` and
/// `write-fonts`, but no subsetter such as `subsetter` or `klippa`). Until one is added, this
/// returns an error, and the rest of the tool (collection, copying, manifest, coverage checks)
/// works unchanged. `name` is the output file name, used only in the message.
pub(super) fn subset_font(
    source: &[u8],
    keep: &BTreeSet<u32>,
    name: &str,
) -> Result<Vec<u8>, AppError> {
    Err(AppError::Encoding(format!(
        "cannot subset {name}: {} source bytes and {} codepoints requested, but no font \
         subsetting crate is available in this build (see font_pack::subset)",
        source.len(),
        keep.len()
    )))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::subset_font;
    use crate::AppError;

    #[test]
    fn reports_the_missing_subsetter_by_output_name() {
        assert!(matches!(
            subset_font(&[0, 1, 2], &BTreeSet::from([0x41]), "sans-sc.ttf"),
            Err(AppError::Encoding(message)) if message.contains("sans-sc.ttf")
        ));
    }
}
