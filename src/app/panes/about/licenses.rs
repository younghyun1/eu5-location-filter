//! Full licence texts shown in the About dialog. The two licences of this crate are embedded
//! from the repository files; the licence of the interface toolkit and the font licence are
//! copied verbatim below. The files wrap their lines at a fixed column, which does not suit a
//! dialog of another width, so each text is reflowed once, on first use, into paragraphs.

use std::sync::LazyLock;

/// MIT licence of the interface toolkit (iced), verbatim.
const ICED_MIT: &str = r##"Copyright 2019 Héctor Ramón, Iced contributors

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
"##;

/// SIL Open Font License 1.1 as published, without the copyright template lines at the head of
/// the published text: those are placeholders, and every font family keeps its own copyright
/// notice in its own files.
const FONTS_OFL: &str = r##"This Font Software is licensed under the SIL Open Font License, Version 1.1.
This license is copied below, and is also available with a FAQ at:
http://scripts.sil.org/OFL


-----------------------------------------------------------
SIL OPEN FONT LICENSE Version 1.1 - 26 February 2007
-----------------------------------------------------------

PREAMBLE
The goals of the Open Font License (OFL) are to stimulate worldwide
development of collaborative font projects, to support the font creation
efforts of academic and linguistic communities, and to provide a free and
open framework in which fonts may be shared and improved in partnership
with others.

The OFL allows the licensed fonts to be used, studied, modified and
redistributed freely as long as they are not sold by themselves. The
fonts, including any derivative works, can be bundled, embedded,
redistributed and/or sold with any software provided that any reserved
names are not used by derivative works. The fonts and derivatives,
however, cannot be released under any other type of license. The
requirement for fonts to remain under this license does not apply
to any document created using the fonts or their derivatives.

DEFINITIONS
"Font Software" refers to the set of files released by the Copyright
Holder(s) under this license and clearly marked as such. This may
include source files, build scripts and documentation.

"Reserved Font Name" refers to any names specified as such after the
copyright statement(s).

"Original Version" refers to the collection of Font Software components as
distributed by the Copyright Holder(s).

"Modified Version" refers to any derivative made by adding to, deleting,
or substituting -- in part or in whole -- any of the components of the
Original Version, by changing formats or by porting the Font Software to a
new environment.

"Author" refers to any designer, engineer, programmer, technical
writer or other person who contributed to the Font Software.

PERMISSION & CONDITIONS
Permission is hereby granted, free of charge, to any person obtaining
a copy of the Font Software, to use, study, copy, merge, embed, modify,
redistribute, and sell modified and unmodified copies of the Font
Software, subject to the following conditions:

1) Neither the Font Software nor any of its individual components,
in Original or Modified Versions, may be sold by itself.

2) Original or Modified Versions of the Font Software may be bundled,
redistributed and/or sold with any software, provided that each copy
contains the above copyright notice and this license. These can be
included either as stand-alone text files, human-readable headers or
in the appropriate machine-readable metadata fields within text or
binary files as long as those fields can be easily viewed by the user.

3) No Modified Version of the Font Software may use the Reserved Font
Name(s) unless explicit written permission is granted by the corresponding
Copyright Holder. This restriction only applies to the primary font name as
presented to the users.

4) The name(s) of the Copyright Holder(s) or the Author(s) of the Font
Software shall not be used to promote, endorse or advertise any
Modified Version, except to acknowledge the contribution(s) of the
Copyright Holder(s) and the Author(s) or with their explicit written
permission.

5) The Font Software, modified or unmodified, in part or in whole,
must be distributed entirely under this license, and must not be
distributed under any other license. The requirement for fonts to
remain under this license does not apply to any document created
using the Font Software.

TERMINATION
This license becomes null and void if any of the above conditions are
not met.

DISCLAIMER
THE FONT SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO ANY WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT
OF COPYRIGHT, PATENT, TRADEMARK, OR OTHER RIGHT. IN NO EVENT SHALL THE
COPYRIGHT HOLDER BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
INCLUDING ANY GENERAL, SPECIAL, INDIRECT, INCIDENTAL, OR CONSEQUENTIAL
DAMAGES, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
FROM, OUT OF THE USE OR INABILITY TO USE THE FONT SOFTWARE OR FROM
OTHER DEALINGS IN THE FONT SOFTWARE.
"##;

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
    fonts: reflow(FONTS_OFL),
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
    use super::{FONTS_OFL, ICED_MIT, reflow, texts};

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
        assert!(FONTS_OFL.contains("SIL OPEN FONT LICENSE Version 1.1"));
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
        assert_eq!(words(&texts().fonts), words(FONTS_OFL));
    }
}
