//! Result types returned by [`run`](super::run).

/// What was written for one output font.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FontOutputReport {
    /// Output file name inside the output directory, for example `plex-sans-kr.ttf`.
    pub name: String,
    /// Source file name inside the source directory.
    pub source_file: String,
    /// Lower-case hexadecimal SHA-256 of the source file.
    pub source_sha256: String,
    /// Version string from the source's `name` table (name ID 5), when it has a readable one.
    pub source_version: Option<String>,
    /// Number of Unicode codepoints the written font maps to a glyph.
    pub codepoints: usize,
    /// Size of the written file in bytes.
    pub size_bytes: u64,
    /// Lower-case hexadecimal SHA-256 of the written file.
    pub output_sha256: String,
    /// Whether the source was copied unchanged because shaping tables must survive.
    pub copied_whole: bool,
    /// Findings that did not stop the run, such as a missing catalog.
    pub notes: Vec<String>,
}

/// Every output written by one run, in generation order.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FontPackReport {
    /// One entry per output file.
    pub outputs: Vec<FontOutputReport>,
}
