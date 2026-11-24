//! Which codepoints each output must cover.
//!
//! Collection is pure: callers read catalog files and packs, this module only turns text into
//! sets. A *required* codepoint makes a run fail when the source font lacks it. An *optional*
//! codepoint is kept when the source has it and is ignored otherwise.

use std::collections::BTreeSet;

use super::spec::Plan;

/// Codepoints one output must or may contain.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct Wanted {
    /// Codepoints that must be present in the written font.
    pub(super) required: BTreeSet<u32>,
    /// Codepoints kept when the source font has them.
    pub(super) optional: BTreeSet<u32>,
}

impl Wanted {
    /// Every codepoint the subsetter should try to keep.
    pub(super) fn keep(&self) -> BTreeSet<u32> {
        self.required.union(&self.optional).copied().collect()
    }
}

/// Whether a character never needs a glyph of its own: controls, spaces and the invisible
/// formatting characters that shaping consumes without drawing.
pub(super) fn is_ignorable(character: char) -> bool {
    character.is_control()
        || character.is_whitespace()
        || matches!(
            u32::from(character),
            0x00AD | 0x061C | 0x200B..=0x200F | 0x202A..=0x202E | 0x2060..=0x2069 | 0xFEFF
        )
}

/// Adds every character of the *values* of a catalog file.
///
/// Keys, comments and blank lines are skipped; the line format is described in
/// `assets/i18n/README.md`. Escapes and placeholders are plain ASCII, so the raw value is enough.
pub(super) fn add_catalog_text(text: &str, out: &mut BTreeSet<u32>) {
    for line in text.lines() {
        let line = line.trim_start();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((_, value)) = line.split_once('=') {
            out.extend(value.chars().map(u32::from));
        }
    }
}

/// Adds every character of the given strings, such as a language pack's string table.
pub(super) fn add_strings<'a>(
    strings: impl IntoIterator<Item = &'a String>,
    out: &mut BTreeSet<u32>,
) {
    for text in strings {
        out.extend(text.chars().map(u32::from));
    }
}

/// Printable ASCII, which every face carries because numbers, identifiers and English words
/// appear inside text of every locale.
pub(super) fn ascii_baseline() -> BTreeSet<u32> {
    (0x20..=0x7E).collect()
}

/// The set every Plex face must contain: Latin, Latin Extended-A, Vietnamese, basic Cyrillic and
/// the punctuation the interface draws, including U+00D7 and the guillemets U+2039 and U+203A.
pub(super) fn plex_core() -> BTreeSet<u32> {
    let mut set = ascii_baseline();
    set.extend(0xA0..=0xFF);
    set.remove(&0xAD);
    set.extend(0x100..=0x17F);
    set.extend([0x1A0, 0x1A1, 0x1AF, 0x1B0]);
    set.extend(0x1EA0..=0x1EF9);
    set.extend(0x400..=0x45F);
    set.extend([0x490, 0x491]);
    set.extend([
        0x2013, 0x2014, 0x2018, 0x2019, 0x201A, 0x201C, 0x201D, 0x201E, 0x2022, 0x2026, 0x2039,
        0x203A, 0x20AC, 0x2212, 0xD7,
    ]);
    set
}

/// Wider ranges kept when the source has them: Latin Extended-B, Latin Extended Additional,
/// extended Cyrillic and general punctuation.
pub(super) fn plex_extra() -> BTreeSet<u32> {
    let mut set: BTreeSet<u32> = (0x180..=0x24F).collect();
    set.extend(0x1E00..=0x1EFF);
    set.extend(0x460..=0x52F);
    set.extend(0x2010..=0x2027);
    set.extend(0x2030..=0x205E);
    set
}

/// Keeps the codepoints that need a glyph.
fn printable(text: &BTreeSet<u32>) -> BTreeSet<u32> {
    text.iter()
        .copied()
        .filter(|&code| char::from_u32(code).is_some_and(|c| !is_ignorable(c)))
        .collect()
}

/// Builds the requirement for one output from the text its locales and packs contribute.
pub(super) fn wanted_for(plan: Plan, text: &BTreeSet<u32>) -> Wanted {
    match plan {
        Plan::Embedded { with_text } => {
            let mut required = plex_core();
            if with_text {
                required.extend(printable(text));
            }
            let optional = plex_extra().difference(&required).copied().collect();
            Wanted { required, optional }
        }
        Plan::Subset => {
            let mut required = ascii_baseline();
            required.extend(printable(text));
            Wanted {
                required,
                optional: BTreeSet::new(),
            }
        }
        Plan::Whole => {
            // Latin, digits and punctuation of these locales render with the embedded face, so
            // only the script's own characters are required of the pack.
            let shared: BTreeSet<u32> = plex_core().union(&plex_extra()).copied().collect();
            let required = printable(text).difference(&shared).copied().collect();
            Wanted {
                required,
                optional: BTreeSet::new(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{add_catalog_text, add_strings, is_ignorable, plex_core, wanted_for};
    use crate::font_pack::spec::Plan;

    fn collect(text: &str) -> BTreeSet<u32> {
        let mut out = BTreeSet::new();
        add_catalog_text(text, &mut out);
        out
    }

    #[test]
    fn catalog_text_reads_values_only() {
        let text = "# comment 漢\n\napp.title = 한글\nrgb.placeholder = #RRGGBB=x\nno equals 字\n  \t\n";
        let found = collect(text);
        assert!(found.contains(&0xD55C), "Hangul from a value");
        assert!(found.contains(&u32::from('#')), "a hash inside a value is text");
        assert!(found.contains(&u32::from('=')), "later equals signs belong to the value");
        assert!(!found.contains(&0x6F22), "comments are skipped");
        assert!(!found.contains(&0x5B57), "lines without an equals sign are skipped");
        assert!(!found.contains(&u32::from('k')), "keys are skipped");
    }

    #[test]
    fn strings_add_every_character() {
        let mut out = BTreeSet::new();
        add_strings(&["가".to_owned(), "ab".to_owned()], &mut out);
        assert_eq!(out, BTreeSet::from([0xAC00, 0x61, 0x62]));
    }

    #[test]
    fn ignorable_characters_need_no_glyph() {
        for c in ['\n', ' ', '\u{200B}', '\u{200D}', '\u{FEFF}', '\u{202E}'] {
            assert!(is_ignorable(c), "{c:?}");
        }
        assert!(!is_ignorable('a'));
        assert!(!is_ignorable('한'));
    }

    #[test]
    fn plex_core_has_the_listed_symbols() {
        let core = plex_core();
        for code in [0xD7, 0x2039, 0x203A, 0x1EA0, 0x1B0, 0x416, 0xE9, 0x20] {
            assert!(core.contains(&code), "U+{code:04X}");
        }
        assert!(!core.contains(&0xAD));
    }

    #[test]
    fn embedded_sans_adds_locale_text_but_mono_does_not() {
        let text = BTreeSet::from([0x15E, 0x1F600, 0x20]);
        let sans = wanted_for(Plan::Embedded { with_text: true }, &text);
        assert!(sans.required.contains(&0x1F600));
        let mono = wanted_for(Plan::Embedded { with_text: false }, &text);
        assert!(!mono.required.contains(&0x1F600));
        assert!(mono.required.contains(&0xD7));
        assert!(mono.optional.contains(&0x1E00));
        assert!(mono.optional.is_disjoint(&mono.required));
    }

    #[test]
    fn subset_requires_text_and_ascii_but_not_whitespace() {
        let text = BTreeSet::from([0xAC00, 0x20, 0x0A, 0x200B]);
        let wanted = wanted_for(Plan::Subset, &text);
        assert!(wanted.required.contains(&0xAC00));
        assert!(wanted.required.contains(&u32::from('A')));
        assert!(!wanted.required.contains(&0x0A));
        assert!(!wanted.required.contains(&0x200B));
        assert!(wanted.optional.is_empty());
    }

    #[test]
    fn whole_requires_only_script_characters() {
        let text = BTreeSet::from([0x627, 0x661, u32::from('A'), 0xD7, 0x200C]);
        let wanted = wanted_for(Plan::Whole, &text);
        assert_eq!(wanted.required, BTreeSet::from([0x627, 0x661]));
    }
}
