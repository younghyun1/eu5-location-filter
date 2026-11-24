//! Reads the catalogs and language packs that contribute text to an output.

use std::collections::BTreeSet;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use super::codepoints::{add_catalog_text, add_strings};
use crate::AppError;
use crate::lang_pack::{GameLanguage, load_pack};

/// Largest catalog file read, far above the real few-kilobyte files.
const MAX_CATALOG_BYTES: u64 = 4 * 1024 * 1024;

/// Text gathered for one output.
#[derive(Debug, Default)]
pub(super) struct Gathered {
    /// Every character that occurs, including whitespace; the caller filters.
    pub(super) codepoints: BTreeSet<u32>,
    /// Findings that did not stop the run.
    pub(super) notes: Vec<String>,
}

/// Collects the characters of the given catalogs and language packs.
///
/// A catalog file that does not exist yet is noted and contributes nothing, because the locale
/// then falls back to English text. A missing or ambiguous language pack is an error: packs are
/// generated before fonts, and a silent gap would drop glyphs.
pub(super) fn gather(
    locales: &[&str],
    languages: &[GameLanguage],
    catalog_dir: &Path,
    lang_dir: &Path,
) -> Result<Gathered, AppError> {
    let mut gathered = Gathered::default();
    for tag in locales {
        let path = catalog_dir.join(format!("{tag}.txt"));
        match read_bounded(&path) {
            Ok(text) => add_catalog_text(&text, &mut gathered.codepoints),
            Err(AppError::Io { source, .. }) if source.kind() == ErrorKind::NotFound => {
                gathered
                    .notes
                    .push(format!("catalog {tag}.txt not found; it adds no codepoints"));
            }
            Err(error) => return Err(error),
        }
    }
    for language in languages {
        let path = find_pack(lang_dir, *language)?;
        let pack = load_pack(&path)?;
        if pack.language != *language {
            return Err(AppError::InvalidData(format!(
                "{} holds {:?} text, expected {:?}",
                path.display(),
                pack.language,
                language
            )));
        }
        add_strings(&pack.strings, &mut gathered.codepoints);
    }
    Ok(gathered)
}

/// Reads a UTF-8 text file after checking its size.
fn read_bounded(path: &Path) -> Result<String, AppError> {
    let size = fs::metadata(path)
        .map_err(|source| AppError::io("inspect", path, source))?
        .len();
    if size > MAX_CATALOG_BYTES {
        return Err(AppError::InvalidData(format!(
            "catalog exceeds its size limit: {}",
            path.display()
        )));
    }
    fs::read_to_string(path).map_err(|source| AppError::io("read", path, source))
}

/// Whether `name` is `<code>.<digits>.bitcode.zst`, the file name of a pack for `code`.
pub(super) fn is_pack_name(name: &str, code: &str) -> bool {
    let Some(rest) = name.strip_prefix(code).and_then(|rest| rest.strip_prefix('.')) else {
        return false;
    };
    rest.strip_suffix(".bitcode.zst")
        .is_some_and(|build| !build.is_empty() && build.bytes().all(|b| b.is_ascii_digit()))
}

/// Finds the single pack file of `language` inside `lang_dir`.
fn find_pack(lang_dir: &Path, language: GameLanguage) -> Result<PathBuf, AppError> {
    let code = language.code();
    let entries = fs::read_dir(lang_dir).map_err(|source| AppError::io("list", lang_dir, source))?;
    let mut found = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| AppError::io("list", lang_dir, source))?;
        if entry
            .file_name()
            .to_str()
            .is_some_and(|name| is_pack_name(name, code))
        {
            found.push(entry.path());
        }
    }
    match (found.pop(), found.is_empty()) {
        (Some(path), true) => Ok(path),
        (Some(_), false) => Err(AppError::InvalidData(format!(
            "several {code} language packs in {}; keep one",
            lang_dir.display()
        ))),
        (None, _) => Err(AppError::MissingFile(
            lang_dir.join(format!("{code}.<build>.bitcode.zst")),
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{gather, is_pack_name};
    use crate::AppError;
    use crate::lang_pack::GameLanguage;

    #[test]
    fn pack_names_need_a_numeric_build() {
        assert!(is_pack_name("ko.26547371.bitcode.zst", "ko"));
        assert!(is_pack_name("pt-br.1.bitcode.zst", "pt-br"));
        assert!(!is_pack_name("ko.bitcode.zst", "ko"));
        assert!(!is_pack_name("ko.x1.bitcode.zst", "ko"));
        assert!(!is_pack_name("ko.1.bitcode.zst.tmp", "ko"));
        assert!(!is_pack_name("kor.1.bitcode.zst", "ko"));
        assert!(!is_pack_name("pt-br.1.bitcode.zst", "pt"));
    }

    #[test]
    fn catalogs_are_read_and_absent_ones_are_noted() -> Result<(), AppError> {
        let root = std::env::temp_dir().join(format!("eu5-font-pack-gather-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).map_err(|e| AppError::io("create", &root, e))?;
        let catalog = root.join("ko-KR.txt");
        fs::write(&catalog, "# note\napp.title = 한\n").map_err(|e| AppError::io("write", &catalog, e))?;
        let gathered = gather(&["ko-KR", "ja-JP"], &[], &root, &root)?;
        let _ = fs::remove_dir_all(&root);
        assert!(gathered.codepoints.contains(&0xD55C));
        assert_eq!(gathered.notes.len(), 1);
        assert!(gathered.notes[0].contains("ja-JP.txt"));
        Ok(())
    }

    #[test]
    fn a_missing_pack_is_an_error() {
        let root = std::env::temp_dir().join("eu5-font-pack-no-such-lang-dir");
        assert!(gather(&[], &[GameLanguage::Korean], &root, &root).is_err());
    }
}
