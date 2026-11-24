//! Build script: stages the committed bundles, the embedded fonts and the desktop language and
//! font packs into `OUT_DIR`, and generates the interface catalog tables.
//!
//! Every staged file has a fixed name under `OUT_DIR` that crate code includes with
//! `include_bytes!` or `include!`; an optional input that is absent is staged as an empty file so
//! the include still compiles and the crate can report the missing asset at run time.

use std::path::{Path, PathBuf};

/// Committed location and index bundles, staged for every target.
const BUNDLES: [&str; 2] = ["eu5-locations.bitcode.zst", "eu5-indexes.bitcode.zst"];

/// Embedded interface faces, staged for every target as `OUT_DIR/fonts/<name>.ttf` from
/// `assets/fonts/<name>.ttf`. Kept in the order of `app::load::fonts::EMBEDDED_FACES`.
const EMBEDDED_FONTS: [&str; 5] = [
    "plex-sans-regular",
    "plex-sans-medium",
    "plex-sans-semibold",
    "plex-mono-regular",
    "plex-mono-medium",
];

/// Script font packs, staged only for the desktop build as `OUT_DIR/fonts/<name>.ttf`. The names
/// equal `FontPack::file_stem` in `src/app/i18n/fonts.rs` and the browser fetch paths.
const FONT_PACKS: [&str; 8] = [
    "plex-sans-kr",
    "plex-sans-jp",
    "sans-sc",
    "sans-tc",
    "noto-sans-arabic",
    "noto-sans-devanagari",
    "noto-sans-bengali",
    "noto-sans-telugu",
];

/// Pack codes in `GameLanguage::PACKED` order; each is staged as `OUT_DIR/lang-<code>.bitcode.zst`.
const PACK_CODES: [&str; 10] = [
    "fr", "de", "es", "pt-br", "pl", "ru", "tr", "ja", "ko", "zh-hans",
];

/// Site locale tags in `Locale::ALL` order (`src/app/i18n/locale.rs`); the generated
/// `CATALOGS` table uses this order, so both lists must change together.
const LOCALE_TAGS: [&str; 27] = [
    "en-US", "ko-KR", "fr-FR", "es-ES", "zh-Hans", "zh-Hant", "ja-JP", "de-DE", "ar", "hi-IN",
    "bn-BD", "te-IN", "pt-BR", "id-ID", "ru-RU", "ur-PK", "vi-VN", "tl-PH", "fa-IR", "it-IT",
    "pl-PL", "sv-SE", "sh", "ro-RO", "nl-NL", "uk-UA", "tr-TR",
];

/// Generated catalog file, included by `src/app/i18n/catalog.rs`.
const CATALOG_FILE: &str = "i18n_catalog.rs";

fn main() {
    if let Err(error) = run() {
        eprintln!("build script failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let Some(output) = std::env::var_os("OUT_DIR") else {
        return Err("Cargo did not set OUT_DIR".to_owned());
    };
    let output = PathBuf::from(output);
    let desktop = std::env::var_os("CARGO_FEATURE_DESKTOP").is_some();
    stage_bundles(&output)?;
    stage_fonts(&output, desktop)?;
    if desktop {
        stage_language_packs(&output)?;
    }
    generate_catalogs(Path::new("assets/i18n"), &output.join(CATALOG_FILE))
}

/// Copies the two committed bundles; both are required.
fn stage_bundles(output: &Path) -> Result<(), String> {
    for name in BUNDLES {
        let source = Path::new("assets").join(name);
        println!("cargo:rerun-if-changed={}", source.display());
        if !source.is_file() {
            return Err(format!(
                "required committed bundle is missing: {}",
                source.display()
            ));
        }
        copy(&source, &output.join(name))?;
    }
    Ok(())
}

/// Stages embedded faces for every target and script packs for the desktop build. A missing
/// source becomes an empty file, which the crate skips when it registers fonts.
fn stage_fonts(output: &Path, desktop: bool) -> Result<(), String> {
    let source_dir = Path::new("assets/fonts");
    println!("cargo:rerun-if-changed={}", source_dir.display());
    let target_dir = output.join("fonts");
    std::fs::create_dir_all(&target_dir)
        .map_err(|error| format!("failed to create {}: {error}", target_dir.display()))?;
    let packs: &[&str] = if desktop { &FONT_PACKS } else { &[] };
    for name in EMBEDDED_FONTS.iter().chain(packs) {
        let file_name = format!("{name}.ttf");
        copy_or_empty(&source_dir.join(&file_name), &target_dir.join(&file_name))?;
    }
    Ok(())
}

/// Stages one pack per packed language from `assets/lang/<code>.<build>.bitcode.zst`. No match
/// stages an empty file, more than one match fails, so the desktop binary embeds exactly the
/// pack that pairs with the committed bundles.
fn stage_language_packs(output: &Path) -> Result<(), String> {
    let source_dir = Path::new("assets/lang");
    println!("cargo:rerun-if-changed={}", source_dir.display());
    let names = list_file_names(source_dir)?;
    for code in PACK_CODES {
        let matches: Vec<&String> = names
            .iter()
            .filter(|name| is_pack_name(name, code))
            .collect();
        let target = output.join(format!("lang-{code}.bitcode.zst"));
        match matches.as_slice() {
            [] => write_empty(&target)?,
            [name] => copy(&source_dir.join(name.as_str()), &target)?,
            _ => {
                return Err(format!(
                    "more than one {code} language pack in {}",
                    source_dir.display()
                ));
            }
        }
    }
    Ok(())
}

/// Matches `<code>.<digits>.bitcode.zst` exactly, so `zh-hans` never matches a shorter code.
fn is_pack_name(name: &str, code: &str) -> bool {
    let Some(rest) = name.strip_prefix(code) else {
        return false;
    };
    let Some(rest) = rest.strip_prefix('.') else {
        return false;
    };
    let Some(digits) = rest.strip_suffix(".bitcode.zst") else {
        return false;
    };
    !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
}

fn list_file_names(directory: &Path) -> Result<Vec<String>, String> {
    if !directory.is_dir() {
        return Ok(Vec::new());
    }
    let entries = std::fs::read_dir(directory)
        .map_err(|error| format!("failed to read {}: {error}", directory.display()))?;
    let mut names = Vec::new();
    for entry in entries {
        let entry =
            entry.map_err(|error| format!("failed to read {}: {error}", directory.display()))?;
        if let Some(name) = entry.file_name().to_str() {
            names.push(name.to_owned());
        }
    }
    Ok(names)
}

fn copy(source: &Path, target: &Path) -> Result<(), String> {
    std::fs::copy(source, target)
        .map(|_| ())
        .map_err(|error| format!("failed to copy {}: {error}", source.display()))
}

fn copy_or_empty(source: &Path, target: &Path) -> Result<(), String> {
    if source.is_file() {
        copy(source, target)
    } else {
        write_empty(target)
    }
}

fn write_empty(target: &Path) -> Result<(), String> {
    std::fs::write(target, [])
        .map_err(|error| format!("failed to write {}: {error}", target.display()))
}

// Interface catalog generation. The format is specified in assets/i18n/README.md; the generated
// file's exact shape is specified in the rustdoc of src/app/i18n/catalog.rs.

/// CLDR plural category written in square brackets after a key.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Category {
    Zero,
    One,
    Two,
    Few,
    Many,
    Other,
}

/// One decoded catalog line.
#[derive(Debug)]
struct Entry {
    /// Dotted key without the plural suffix.
    key: String,
    /// Plural category, when the line carries one.
    category: Option<Category>,
    /// Value with backslash escapes decoded; brace syntax is kept for placeholder values and
    /// decoded for values without placeholders.
    value: String,
    /// One-based source line for error messages.
    line: usize,
}

/// One parsed locale file.
#[derive(Debug)]
struct Catalog {
    /// Site locale tag, one of `LOCALE_TAGS`.
    tag: &'static str,
    /// Entries in file order.
    entries: Vec<Entry>,
}

/// Reads every catalog, checks the README rules, and writes the generated tables. A locale
/// without a file is generated as `None` and falls back to English at run time; a `.txt` file
/// with an unknown tag is an error.
fn generate_catalogs(directory: &Path, output: &Path) -> Result<(), String> {
    println!("cargo:rerun-if-changed={}", directory.display());
    let _ = (output, LOCALE_TAGS);
    Err("interface catalog generation is not implemented yet".to_owned())
}

/// Parses one file per the README line rules, decoding escapes and checking keys and braces.
fn parse_catalog(tag: &'static str, text: &str) -> Result<Catalog, String> {
    let _ = text;
    Err(format!("catalog parsing is not implemented yet for {tag}"))
}

/// Decodes `\n`, `\s` and `\\`; any other backslash sequence is an error.
fn decode_escapes(raw: &str, line: usize) -> Result<String, String> {
    let _ = raw;
    Err(format!("escape decoding is not implemented yet (line {line})"))
}

/// Renders the generated Rust source from English and the other catalogs in `LOCALE_TAGS` order.
fn render(english: &Catalog, others: &[Option<Catalog>]) -> String {
    let _ = (english, others);
    String::new()
}

/// Enum variant for a key: split at `.` and `_`, capitalize each part, join
/// (`grid.show_details` becomes `GridShowDetails`).
fn variant_name(key: &str) -> String {
    let mut name = String::with_capacity(key.len());
    for part in key.split(['.', '_']) {
        let mut characters = part.chars();
        if let Some(first) = characters.next() {
            name.extend(first.to_uppercase());
            name.push_str(characters.as_str());
        }
    }
    name
}
