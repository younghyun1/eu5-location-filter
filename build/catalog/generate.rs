//! Reads the catalog directory, runs every check, and writes the generated tables.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::check::{check_catalog, check_variants};
use super::locales::LOCALE_TAGS;
use super::model::Catalog;
use super::parse::parse_catalog;
use super::render::render;

/// Most errors reported in one build, so a badly broken file stays readable.
const MAX_ERRORS: usize = 50;

/// Reads every catalog, checks the README rules, and writes the generated tables. A locale
/// without a file is generated as `None` and falls back to English at run time; a `.txt` file
/// with an unknown tag, a missing English catalog, or any parity failure is an error.
pub fn generate_catalogs(directory: &Path, output: &Path) -> Result<(), String> {
    println!("cargo:rerun-if-changed={}", directory.display());
    let files = catalog_files(directory)?;
    let mut errors = Vec::new();
    let mut catalogs: Vec<Option<Catalog>> = Vec::with_capacity(LOCALE_TAGS.len());
    for tag in LOCALE_TAGS {
        let Some(path) = files.get(tag) else {
            catalogs.push(None);
            continue;
        };
        println!("cargo:rerun-if-changed={}", path.display());
        let text = std::fs::read_to_string(path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
        match parse_catalog(tag, path.display().to_string(), &text) {
            Ok(catalog) => catalogs.push(Some(catalog)),
            Err(found) => {
                errors.extend(found);
                catalogs.push(None);
            }
        }
    }
    match catalogs.first().and_then(Option::as_ref) {
        Some(english) => {
            errors.extend(check_variants(english));
            for catalog in catalogs.iter().flatten() {
                errors.extend(check_catalog(english, catalog));
            }
        }
        None if errors.is_empty() => errors.push(format!(
            "{}: the English catalog en-US.txt is required",
            directory.display()
        )),
        None => {}
    }
    if !errors.is_empty() {
        return Err(join_errors(errors));
    }
    let source = render(&catalogs)?;
    std::fs::write(output, source)
        .map_err(|error| format!("failed to write {}: {error}", output.display()))
}

/// Maps each `<tag>.txt` in the directory to its path; other extensions are ignored and a
/// `.txt` file with an unknown tag is an error.
fn catalog_files(directory: &Path) -> Result<BTreeMap<&'static str, PathBuf>, String> {
    let entries = std::fs::read_dir(directory)
        .map_err(|error| format!("failed to read {}: {error}", directory.display()))?;
    let mut files = BTreeMap::new();
    let mut unknown = Vec::new();
    for entry in entries {
        let entry =
            entry.map_err(|error| format!("failed to read {}: {error}", directory.display()))?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        let Some(stem) = name.strip_suffix(".txt") else {
            continue;
        };
        match LOCALE_TAGS.iter().find(|tag| **tag == stem) {
            Some(tag) => {
                files.insert(*tag, entry.path());
            }
            None => unknown.push(format!(
                "{}: `{name}` is not a supported locale tag",
                directory.display()
            )),
        }
    }
    if unknown.is_empty() {
        Ok(files)
    } else {
        unknown.sort();
        Err(join_errors(unknown))
    }
}

fn join_errors(mut errors: Vec<String>) -> String {
    let total = errors.len();
    errors.truncate(MAX_ERRORS);
    if total > MAX_ERRORS {
        errors.push(format!("and {} more", total - MAX_ERRORS));
    }
    format!("interface catalogs are invalid:\n{}", errors.join("\n"))
}
