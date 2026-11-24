//! Reads the Unicode character map and the version string from font bytes.

use std::collections::BTreeSet;

use crate::AppError;

/// Returns every Unicode codepoint the font maps to a real glyph.
///
/// All Unicode `cmap` subtables are merged. A mapping to glyph 0 is the missing-glyph box, so it
/// does not count as coverage. `label` names the font in errors.
pub(super) fn font_codepoints(data: &[u8], label: &str) -> Result<BTreeSet<u32>, AppError> {
    let face = ttf_parser::Face::parse(data, 0)
        .map_err(|error| AppError::parse(label, 0, format!("unreadable font: {error}")))?;
    let Some(cmap) = face.tables().cmap.as_ref() else {
        return Err(AppError::parse(label, 0, "font has no cmap table"));
    };
    let mut found = BTreeSet::new();
    for subtable in cmap.subtables {
        if !subtable.is_unicode() {
            continue;
        }
        subtable.codepoints(|code| {
            if subtable.glyph_index(code).is_some_and(|glyph| glyph.0 != 0) {
                found.insert(code);
            }
        });
    }
    Ok(found)
}

/// Returns the version string (`name` ID 5) of a font, or `None` when it has no readable one.
pub(super) fn font_version(data: &[u8]) -> Option<String> {
    let face = ttf_parser::Face::parse(data, 0).ok()?;
    face.names()
        .into_iter()
        .filter(|name| name.name_id == ttf_parser::name_id::VERSION)
        .find_map(|name| name.to_string())
}

#[cfg(test)]
mod tests {
    use super::{font_codepoints, font_version};
    use crate::AppError;

    #[test]
    fn garbage_is_rejected_with_the_label() {
        assert!(matches!(
            font_codepoints(b"not a font", "bad.ttf"),
            Err(AppError::Parse { source_name, .. }) if source_name == "bad.ttf"
        ));
        assert!(font_codepoints(&[], "empty.ttf").is_err());
    }

    #[test]
    fn garbage_has_no_version() {
        assert_eq!(font_version(b"not a font"), None);
    }
}
