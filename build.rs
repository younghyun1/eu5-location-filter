//! Build script: stages the committed bundles, the embedded fonts and the desktop language and
//! font packs into `OUT_DIR`, and generates the interface catalog tables.
//!
//! Every staged file has a fixed name under `OUT_DIR` that crate code includes with
//! `include_bytes!` or `include!`; an optional input that is absent is staged as an empty file so
//! the include still compiles and the crate can report the missing asset at run time.

use std::path::{Path, PathBuf};

#[path = "build/catalog/mod.rs"]
mod catalog;

/// Committed location and index bundles, staged for every target.
const BUNDLES: [&str; 2] = ["eu5-locations.bitcode.zst", "eu5-indexes.bitcode.zst"];

/// Embedded interface faces, staged for every target as `OUT_DIR/fonts/<name>.ttf` from
/// `assets/fonts/<name>.ttf`. Kept in the order of `app::load::fonts::EMBEDDED_FACES`.
const EMBEDDED_FONTS: [&str; 6] = [
    "plex-sans-regular",
    "plex-sans-medium",
    "plex-sans-semibold",
    "plex-mono-regular",
    "plex-mono-medium",
    "latin-fallback",
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
    catalog::generate_catalogs(
        Path::new("assets/i18n"),
        &output.join(catalog::CATALOG_FILE),
    )
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
