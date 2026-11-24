//! The font-pack command: collect, subset or copy, verify, write.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::cmap::{font_codepoints, font_version};
use super::codepoints::wanted_for;
use super::coverage::verify;
use super::manifest::{MANIFEST_NAME, render, write_atomic};
use super::report::{FontOutputReport, FontPackReport};
use super::sha256::sha256_hex;
use super::sources::{Gathered, gather};
use super::spec::{OUTPUTS, OutputSpec, PRIMARY_SANS_SOURCE, Plan};
use super::subset::subset_font;
use crate::AppError;
use crate::lang_pack::MAX_PACK_COMPRESSED_SIZE;

/// Largest source font read; the biggest real source (IBM Plex Sans SC) is far below this.
const MAX_SOURCE_BYTES: u64 = 64 * 1024 * 1024;

/// Directories used by [`run`].
#[derive(Clone, Debug)]
pub struct FontPackArgs {
    /// Directory holding the source fonts under the file names listed in `assets/fonts/README.md`.
    pub source_dir: PathBuf,
    /// Directory that receives the outputs and `manifest.json`, normally `assets/fonts`.
    pub output_dir: PathBuf,
    /// Directory of interface catalogs (`<tag>.txt`), normally `assets/i18n`.
    pub catalog_dir: PathBuf,
    /// Directory of language packs (`<code>.<build>.bitcode.zst`), normally `assets/lang`.
    pub lang_dir: PathBuf,
}

/// One finished output waiting to be written.
struct Built {
    report: FontOutputReport,
    bytes: Vec<u8>,
}

/// Generates every font, checks coverage, then writes the outputs and the manifest.
///
/// Nothing is written until all outputs have been built and verified, so a failure leaves the
/// previous files and manifest untouched. Each file is replaced atomically.
pub fn run(args: &FontPackArgs) -> Result<FontPackReport, AppError> {
    let mut embedded_text: Option<Gathered> = None;
    let mut built = Vec::with_capacity(OUTPUTS.len());
    for spec in &OUTPUTS {
        built.push(build_one(args, spec, &mut embedded_text)?);
    }
    fs::create_dir_all(&args.output_dir)
        .map_err(|source| AppError::io("create", &args.output_dir, source))?;
    for item in &built {
        write_atomic(&args.output_dir.join(&item.report.name), &item.bytes)?;
    }
    let report = FontPackReport {
        outputs: built.into_iter().map(|item| item.report).collect(),
    };
    write_atomic(
        &args.output_dir.join(MANIFEST_NAME),
        render(&report).as_bytes(),
    )?;
    Ok(report)
}

fn build_one(
    args: &FontPackArgs,
    spec: &OutputSpec,
    embedded_text: &mut Option<Gathered>,
) -> Result<Built, AppError> {
    let source = read_source(&args.source_dir.join(spec.source))?;
    let source_codepoints = font_codepoints(&source, spec.source)?;
    let mut notes = Vec::new();
    let mut text = text_for(args, spec, embedded_text, &mut notes)?;
    if spec.plan == Plan::LatinFallback {
        // Only what the primary face cannot draw belongs in the fallback face.
        let primary = read_source(&args.source_dir.join(PRIMARY_SANS_SOURCE))?;
        let covered = font_codepoints(&primary, PRIMARY_SANS_SOURCE)?;
        text.retain(|code| !covered.contains(code));
    }
    let mut wanted = wanted_for(spec.plan, &text);
    if spec.plan == Plan::Subset {
        waive_stray_codepoints(spec, &mut wanted.required, &source_codepoints, &mut notes);
    }
    // Checking the source first names the font to replace when it lacks a script's characters.
    verify(spec.source, &wanted.required, &source_codepoints)?;
    let source_sha256 = sha256_hex(&source);
    let source_version = font_version(&source);
    let (bytes, copied_whole) = match spec.plan {
        Plan::Whole | Plan::Embedded { .. } => (source, true),
        Plan::LatinFallback | Plan::Subset => (
            subset_font(&source, &wanted.keep(), spec.name, spec.rename)?,
            false,
        ),
    };
    if !spec.plan.is_embedded()
        && u64::try_from(bytes.len()).map_or(true, |size| size > MAX_PACK_COMPRESSED_SIZE)
    {
        return Err(AppError::InvalidData(format!(
            "{} exceeds the {MAX_PACK_COMPRESSED_SIZE} byte fetch limit",
            spec.name
        )));
    }
    let written = font_codepoints(&bytes, spec.name)?;
    verify(spec.name, &wanted.required, &written)?;
    let report = FontOutputReport {
        name: spec.name.to_owned(),
        source_file: spec.source.to_owned(),
        source_sha256,
        source_version,
        codepoints: written.len(),
        size_bytes: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
        output_sha256: sha256_hex(&bytes),
        copied_whole,
        notes,
    };
    Ok(Built { report, bytes })
}

/// Most stray codepoints a CJK source may lack before the run fails.
const MAX_WAIVED_CODEPOINTS: usize = 16;

/// Game text occasionally contains a character from another script's repertoire (a Simplified
/// Chinese form inside Japanese text, for example). A handful of such characters must not block
/// the whole pack: they are dropped from the requirement and recorded in the manifest, where a
/// reviewer can see them. More than [`MAX_WAIVED_CODEPOINTS`] means the wrong source font.
fn waive_stray_codepoints(
    spec: &OutputSpec,
    required: &mut BTreeSet<u32>,
    source_codepoints: &BTreeSet<u32>,
    notes: &mut Vec<String>,
) {
    let missing: Vec<u32> = required.difference(source_codepoints).copied().collect();
    if missing.is_empty() || missing.len() > MAX_WAIVED_CODEPOINTS {
        return;
    }
    let listed: Vec<String> = missing.iter().map(|code| format!("U+{code:04X}")).collect();
    notes.push(format!(
        "{} lacks {} codepoint(s) used by the text, shown as missing glyphs: {}",
        spec.source,
        missing.len(),
        listed.join(", ")
    ));
    for code in missing {
        required.remove(&code);
    }
}

/// Collects the catalog and pack text an output needs. The text of the embedded-font locales is
/// gathered once and shared by the three Sans faces, because loading seven packs is not free.
fn text_for(
    args: &FontPackArgs,
    spec: &OutputSpec,
    embedded_text: &mut Option<Gathered>,
    notes: &mut Vec<String>,
) -> Result<BTreeSet<u32>, AppError> {
    match spec.plan {
        Plan::Embedded { with_text: false } => Ok(BTreeSet::new()),
        Plan::Embedded { with_text: true } | Plan::LatinFallback => {
            if embedded_text.is_none() {
                *embedded_text = Some(gather(
                    spec.locales,
                    spec.languages,
                    &args.catalog_dir,
                    &args.lang_dir,
                )?);
            }
            Ok(embedded_text
                .as_ref()
                .map_or_else(BTreeSet::new, |gathered| {
                    notes.extend(gathered.notes.iter().cloned());
                    gathered.codepoints.clone()
                }))
        }
        Plan::Subset | Plan::Whole => {
            let gathered = gather(
                spec.locales,
                spec.languages,
                &args.catalog_dir,
                &args.lang_dir,
            )?;
            notes.extend(gathered.notes);
            Ok(gathered.codepoints)
        }
    }
}

fn read_source(path: &Path) -> Result<Vec<u8>, AppError> {
    let size = fs::metadata(path)
        .map_err(|source| {
            if source.kind() == std::io::ErrorKind::NotFound {
                AppError::MissingFile(path.to_path_buf())
            } else {
                AppError::io("inspect", path, source)
            }
        })?
        .len();
    if size > MAX_SOURCE_BYTES {
        return Err(AppError::InvalidData(format!(
            "source font exceeds its size limit: {}",
            path.display()
        )));
    }
    fs::read(path).map_err(|source| AppError::io("read", path, source))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{FontPackArgs, run};
    use crate::AppError;

    fn args(root: &std::path::Path) -> FontPackArgs {
        FontPackArgs {
            source_dir: root.join("source"),
            output_dir: root.join("out"),
            catalog_dir: root.join("i18n"),
            lang_dir: root.join("lang"),
        }
    }

    #[test]
    fn a_missing_source_font_is_reported_and_nothing_is_written() -> Result<(), AppError> {
        let root = std::env::temp_dir().join(format!("eu5-font-pack-run-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("source")).map_err(|e| AppError::io("create", &root, e))?;
        let result = run(&args(&root));
        let wrote = root.join("out").exists();
        let _ = fs::remove_dir_all(&root);
        assert!(
            matches!(result, Err(AppError::MissingFile(path)) if path.ends_with("IBMPlexSans-Regular.ttf"))
        );
        assert!(
            !wrote,
            "no output directory is created before every font is built"
        );
        Ok(())
    }

    #[test]
    fn an_unreadable_source_font_is_a_parse_error() -> Result<(), AppError> {
        let root = std::env::temp_dir().join(format!("eu5-font-pack-bad-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("source")).map_err(|e| AppError::io("create", &root, e))?;
        let font = root.join("source/IBMPlexSans-Regular.ttf");
        fs::write(&font, b"not a font").map_err(|e| AppError::io("write", &font, e))?;
        let result = run(&args(&root));
        let _ = fs::remove_dir_all(&root);
        assert!(matches!(result, Err(AppError::Parse { .. })));
        Ok(())
    }
}
