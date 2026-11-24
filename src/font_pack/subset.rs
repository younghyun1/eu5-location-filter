//! Font subsetting, isolated in one function.

use std::collections::BTreeSet;

use fontcull_klippa::{Plan, SubsetFlags, subset_font as klippa_subset};
use fontcull_read_fonts::collections::IntSet;
use fontcull_skrifa::{FontRef, GlyphId, Tag};
use fontcull_write_fonts::types::NameId;

use crate::AppError;

/// Reduces `source` to the glyphs that `keep` maps and returns the new font file.
///
/// The result keeps the family and style names, vertical metrics and the `cmap` entries for
/// every codepoint of `keep` that the source maps. Codepoints of `keep` that the source lacks
/// are skipped silently; the caller verifies the required ones afterwards. Glyph outlines and
/// the layout tables that the kept glyphs need (`GSUB`, `GPOS`, `GDEF`) survive, because
/// Hangul, kana and Han text is shaped with them. A variable source must be instanced to its
/// default regular weight first. `name` is the output file name, used only in messages.
pub(super) fn subset_font(
    source: &[u8],
    keep: &BTreeSet<u32>,
    name: &str,
) -> Result<Vec<u8>, AppError> {
    let font = match FontRef::new(source) {
        Ok(font) => font,
        Err(error) => {
            return Err(AppError::Encoding(format!(
                "cannot subset {name}: unreadable source font: {error:?}"
            )));
        }
    };
    let mut unicodes: IntSet<u32> = IntSet::empty();
    for code in keep {
        unicodes.insert(*code);
    }
    // The family, style, full and PostScript names (and their typographic variants) must
    // survive: the application selects faces by family name, so a nameless subset would never
    // be chosen.
    let mut name_ids: IntSet<NameId> = IntSet::empty();
    for id in [0_u16, 1, 2, 3, 4, 5, 6, 13, 14, 16, 17] {
        name_ids.insert(NameId::new(id));
    }
    let name_languages: IntSet<u16> = IntSet::all();
    let no_glyphs: IntSet<GlyphId> = IntSet::empty();
    let no_dropped_tables: IntSet<Tag> = IntSet::empty();
    let scripts: IntSet<Tag> = IntSet::all();
    let features: IntSet<Tag> = IntSet::all();
    let plan = Plan::new(
        &no_glyphs,
        &unicodes,
        &font,
        SubsetFlags::default(),
        &no_dropped_tables,
        &scripts,
        &features,
        &name_ids,
        &name_languages,
    );
    match klippa_subset(&font, &plan) {
        Ok(bytes) => Ok(bytes),
        Err(error) => Err(AppError::Encoding(format!(
            "cannot subset {name} to {} codepoints: {error:?}",
            keep.len()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::subset_font;
    use crate::AppError;

    #[test]
    fn reports_an_unreadable_source_by_output_name() {
        assert!(matches!(
            subset_font(&[0, 1, 2], &BTreeSet::from([0x41]), "sans-sc.ttf"),
            Err(AppError::Encoding(message)) if message.contains("sans-sc.ttf")
        ));
    }
}
