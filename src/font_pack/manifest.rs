//! Manifest rendering and crash-safe file writes.

use std::fs;
use std::path::Path;

use super::report::FontPackReport;
use crate::AppError;

/// Name of the manifest inside the output directory.
pub(super) const MANIFEST_NAME: &str = "manifest.json";

/// Escapes a string for use inside a JSON string literal.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if u32::from(c) < 0x20 => out.push_str(&format!("\\u{:04x}", u32::from(c))),
            c => out.push(c),
        }
    }
    out
}

fn string(text: &str) -> String {
    format!("\"{}\"", escape(text))
}

/// Renders the report as pretty JSON with a trailing newline.
pub(super) fn render(report: &FontPackReport) -> String {
    let mut out = String::from("{\n  \"format\": 1,\n  \"outputs\": [");
    for (index, output) in report.outputs.iter().enumerate() {
        out.push_str(if index == 0 {
            "\n    {\n"
        } else {
            ",\n    {\n"
        });
        let version = output
            .source_version
            .as_deref()
            .map_or_else(|| "null".to_owned(), string);
        let notes: Vec<String> = output.notes.iter().map(|note| string(note)).collect();
        out.push_str(&format!("      \"name\": {},\n", string(&output.name)));
        out.push_str(&format!(
            "      \"source\": {},\n",
            string(&output.source_file)
        ));
        out.push_str(&format!(
            "      \"source_sha256\": {},\n",
            string(&output.source_sha256)
        ));
        out.push_str(&format!("      \"source_version\": {version},\n"));
        out.push_str(&format!("      \"codepoints\": {},\n", output.codepoints));
        out.push_str(&format!("      \"size_bytes\": {},\n", output.size_bytes));
        out.push_str(&format!(
            "      \"sha256\": {},\n",
            string(&output.output_sha256)
        ));
        out.push_str(&format!(
            "      \"copied_whole\": {},\n",
            output.copied_whole
        ));
        out.push_str(&format!("      \"notes\": [{}]\n    }}", notes.join(", ")));
    }
    out.push_str(if report.outputs.is_empty() {
        "]\n}\n"
    } else {
        "\n  ]\n}\n"
    });
    out
}

/// Writes `bytes` to a sibling temporary file and renames it over `path`, so an interrupted run
/// never leaves a truncated font behind.
pub(super) fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), AppError> {
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(".tmp");
    let temporary = std::path::PathBuf::from(temporary);
    fs::write(&temporary, bytes).map_err(|source| AppError::io("write", &temporary, source))?;
    fs::rename(&temporary, path).map_err(|source| AppError::io("replace", path, source))
}

#[cfg(test)]
mod tests {
    use super::{escape, render};
    use crate::font_pack::report::{FontOutputReport, FontPackReport};

    fn sample(version: Option<&str>) -> FontOutputReport {
        FontOutputReport {
            name: "plex-sans-kr.ttf".to_owned(),
            source_file: "IBMPlexSansKR-Regular.ttf".to_owned(),
            source_sha256: "ab".repeat(32),
            source_version: version.map(str::to_owned),
            codepoints: 2350,
            size_bytes: 1_234_567,
            output_sha256: "cd".repeat(32),
            copied_whole: false,
            notes: vec!["catalog \"x\" missing".to_owned()],
        }
    }

    #[test]
    fn escapes_quotes_backslashes_and_controls() {
        assert_eq!(escape("a\"b\\c\nd\u{1}"), "a\\\"b\\\\c\\nd\\u0001");
        assert_eq!(escape("한"), "한");
    }

    #[test]
    fn renders_one_object_per_output() {
        let text = render(&FontPackReport {
            outputs: vec![sample(Some("Version 1.1")), sample(None)],
        });
        assert!(text.starts_with("{\n  \"format\": 1,"));
        assert!(text.ends_with("\n  ]\n}\n"));
        assert_eq!(text.matches("\"name\": \"plex-sans-kr.ttf\"").count(), 2);
        assert!(text.contains("\"source_version\": \"Version 1.1\""));
        assert!(text.contains("\"source_version\": null"));
        assert!(text.contains("\"size_bytes\": 1234567"));
        assert!(text.contains("catalog \\\"x\\\" missing"));
    }

    #[test]
    fn renders_an_empty_report() {
        assert_eq!(
            render(&FontPackReport::default()),
            "{\n  \"format\": 1,\n  \"outputs\": []\n}\n"
        );
    }
}
