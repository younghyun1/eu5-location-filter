//! Full licence texts shown in the About dialog. The two licences of this crate are embedded
//! from the repository files, and the licence of the interface toolkit and the font licences
//! from `assets/licenses/`. The files wrap their lines at a fixed column, which does not suit a
//! dialog of another width, so each text is reflowed once, on first use, into paragraphs.

use std::sync::LazyLock;

/// MIT licence of the interface toolkit (iced), as shipped with the iced crate.
const ICED_MIT: &str = include_str!("../../../../assets/licenses/iced-MIT.txt");

/// SIL Open Font License 1.1 with the IBM Plex copyright notice and its reserved font name, as
/// published by IBM. The embedded Plex faces are unmodified; the Korean, Japanese and Chinese
/// packs are renamed subsets of the Plex families, as the reserved name requires.
const PLEX_OFL: &str = include_str!("../../../../assets/licenses/ibm-plex-OFL.txt");

/// The same licence with the copyright notice of the Noto fonts, as published by the Noto
/// project.
const NOTO_OFL: &str = include_str!("../../../../assets/licenses/noto-OFL.txt");

/// Licence texts ready to draw.
pub(super) struct Licenses {
    /// MIT licence of this crate.
    pub(super) source_mit: String,
    /// Apache-2.0 licence of this crate.
    pub(super) source_apache: String,
    /// MIT licence of iced.
    pub(super) iced: String,
    /// SIL Open Font License 1.1.
    pub(super) fonts: String,
}

/// The reflowed texts; four strings built once, so the cache is bounded by construction.
static LICENSES: LazyLock<Licenses> = LazyLock::new(|| Licenses {
    source_mit: reflow(include_str!("../../../../LICENSE-MIT")),
    source_apache: reflow(include_str!("../../../../LICENSE-APACHE")),
    iced: reflow(ICED_MIT),
    fonts: {
        let mut text = reflow(PLEX_OFL);
        text.push_str("\n\n");
        text.push_str(&reflow(NOTO_OFL));
        text
    },
});

/// The licence texts.
#[must_use]
pub(super) fn texts() -> &'static Licenses {
    &LICENSES
}

/// Joins the hard-wrapped lines of each paragraph into one line and keeps indented lines, such
/// as numbered clauses, on their own lines. Blank lines separate paragraphs and runs of them
/// collapse into one.
#[must_use]
pub(super) fn reflow(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    // Whether the previous line was unindented text that the next unindented line continues.
    let mut flowing = false;
    for line in source.lines() {
        let line = line.trim_end();
        if line.trim().is_empty() {
            if !out.is_empty() && !out.ends_with("\n\n") {
                out.push_str("\n\n");
            }
            flowing = false;
            continue;
        }
        let indented = line.starts_with(char::is_whitespace);
        if flowing && !indented {
            out.push(' ');
            out.push_str(line);
        } else {
            if !out.is_empty() && !out.ends_with('\n') {
                out.push('\n');
            }
            out.push_str(line);
        }
        flowing = !indented;
    }
    while out.ends_with('\n') {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{ICED_MIT, NOTO_OFL, PLEX_OFL, reflow, texts};

    #[test]
    fn wrapped_lines_join_and_paragraphs_stay_apart() {
        let source = "first line\ncontinues here\n\n\n\nsecond paragraph\n";
        assert_eq!(
            reflow(source),
            "first line continues here\n\nsecond paragraph"
        );
    }

    #[test]
    fn indented_clauses_keep_their_own_lines() {
        let source = "Terms:\n   1. One\n   2. Two\nafter\ntext\n";
        assert_eq!(reflow(source), "Terms:\n   1. One\n   2. Two\nafter text");
    }

    #[test]
    fn empty_input_stays_empty() {
        assert_eq!(reflow(""), "");
        assert_eq!(reflow("\n\n  \n"), "");
    }

    #[test]
    fn the_embedded_texts_are_the_expected_licences() {
        assert!(ICED_MIT.contains("Permission is hereby granted"));
        assert!(PLEX_OFL.contains("SIL OPEN FONT LICENSE Version 1.1"));
        assert!(PLEX_OFL.contains("Reserved Font Name"));
        assert!(NOTO_OFL.contains("The Noto Project Authors"));
        let all = texts();
        assert!(all.source_mit.contains("MIT"));
        assert!(all.source_apache.contains("Apache License"));
        assert!(all.iced.contains("Iced contributors"));
        assert!(all.fonts.contains("PREAMBLE"));
    }

    #[test]
    fn reflowing_loses_no_words() {
        let words = |text: &str| text.split_whitespace().count();
        assert_eq!(words(&texts().iced), words(ICED_MIT));
        assert_eq!(words(&texts().fonts), words(PLEX_OFL) + words(NOTO_OFL));
    }
}
