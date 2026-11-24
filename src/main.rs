#![cfg_attr(windows, windows_subsystem = "windows")]

//! Desktop entry point: the iced interface by default, plus offline import, reindex, diff and
//! font-pack commands.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use eu5_location_filter::filter::FilterEngine;
use eu5_location_filter::font_pack::{self, FontPackArgs};
use eu5_location_filter::import::{self, ImportProgress, import_game};
use eu5_location_filter::lang_pack::{GameLanguage, Labels, ValidatedLanguagePack};
use eu5_location_filter::model::Dataset;
use eu5_location_filter::{AppError, app, bundle_diff, index_storage, steam, storage};

const DEFAULT_DATA_FILE: &str = "assets/eu5-locations.bitcode.zst";
const DEFAULT_INDEX_FILE: &str = "assets/eu5-indexes.bitcode.zst";

#[derive(Debug, Parser)]
#[command(
    name = "eu5-location-filter",
    version = env!("CARGO_PKG_VERSION"),
    about = "Inspect and filter Europa Universalis V map locations"
)]
struct Cli {
    #[arg(long, global = true)]
    data_file: Option<PathBuf>,
    #[arg(long, global = true)]
    index_file: Option<PathBuf>,
    #[arg(long, global = true)]
    game_dir: Option<PathBuf>,
    /// Directory holding the per-language packs.
    #[arg(long, global = true, default_value = "assets/lang")]
    lang_dir: PathBuf,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Rebuild the query index and every language pack from existing bundles, without reading
    /// EU5 files.
    Reindex {
        /// Replace an existing index bundle.
        #[arg(long)]
        force: bool,
    },
    /// Compare two location bundles field by field, keyed by location identifier, then their
    /// game text per language.
    Diff {
        /// Older bundle.
        #[arg(long)]
        old: PathBuf,
        /// Newer bundle.
        #[arg(long)]
        new: PathBuf,
        /// Language packs of the older bundle.
        #[arg(long)]
        old_lang_dir: Option<PathBuf>,
        /// Language packs of the newer bundle.
        #[arg(long)]
        new_lang_dir: Option<PathBuf>,
        /// Comma-separated language codes to compare; all when omitted.
        #[arg(long, value_delimiter = ',', value_parser = parse_language)]
        languages: Vec<GameLanguage>,
    },
    /// Import a vanilla EU5 installation and its language packs without opening the GUI.
    Import {
        /// Replace an existing valid data file, index and packs.
        #[arg(long)]
        force: bool,
        /// Comma-separated language codes to build packs for; all when omitted.
        #[arg(long, value_delimiter = ',', value_parser = parse_language)]
        languages: Vec<GameLanguage>,
    },
    /// Generate the lazy script font packs from local source fonts.
    Fonts {
        /// Directory holding the source font files.
        #[arg(long)]
        source_dir: PathBuf,
        /// Directory the font packs are written to.
        #[arg(long, default_value = "assets/fonts")]
        output_dir: PathBuf,
        /// Directory of the interface catalogs whose characters the packs must cover.
        #[arg(long, default_value = "assets/i18n")]
        catalog_dir: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            let _ = error.print();
            return ExitCode::from(2);
        }
    };
    let data_file = cli
        .data_file
        .clone()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_DATA_FILE));
    let index_file = cli
        .index_file
        .clone()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_INDEX_FILE));
    let result = match cli.command {
        Some(Command::Reindex { force }) => {
            run_reindex(&data_file, &index_file, &cli.lang_dir, force)
        }
        Some(Command::Diff {
            old,
            new,
            old_lang_dir,
            new_lang_dir,
            languages,
        }) => run_diff(
            &old,
            &new,
            old_lang_dir.as_deref(),
            new_lang_dir.as_deref(),
            &languages,
        ),
        Some(Command::Import { force, languages }) => run_import(
            &data_file,
            &index_file,
            cli.game_dir.as_deref(),
            &cli.lang_dir,
            &languages,
            force,
        ),
        Some(Command::Fonts {
            source_dir,
            output_dir,
            catalog_dir,
        }) => run_fonts(source_dir, output_dir, catalog_dir, cli.lang_dir),
        None => app::run(cli.data_file, cli.index_file, cli.game_dir),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(1)
        }
    }
}

/// Parses one `GameLanguage::code` for clap, which needs a `String` error.
fn parse_language(value: &str) -> Result<GameLanguage, String> {
    value
        .parse::<GameLanguage>()
        .map_err(|error| error.to_string())
}

fn print_progress(progress: ImportProgress) {
    if progress.total > 0 {
        eprintln!(
            "{} ({}/{})",
            progress.stage.label(),
            progress.current,
            progress.total
        );
    } else {
        eprintln!("{}", progress.stage.label());
    }
}

/// Prints the bundle report, then one text section per language: English always, the others
/// only when at least one side names a pack directory.
fn run_diff(
    old_path: &Path,
    new_path: &Path,
    old_lang_dir: Option<&Path>,
    new_lang_dir: Option<&Path>,
    languages: &[GameLanguage],
) -> Result<(), AppError> {
    let old = storage::load_dataset(old_path)?;
    let new = storage::load_dataset(new_path)?;
    let mut output = bundle_diff::diff(&old, &new).report();
    let english = bundle_diff::diff_text(
        GameLanguage::English,
        Labels::new(&old, None),
        Labels::new(&new, None),
    );
    output.push_str(&english.report());
    if old_lang_dir.is_some() || new_lang_dir.is_some() {
        for language in import::selected_languages(languages) {
            if language.is_embedded() {
                continue;
            }
            let old_pack = side_pack(&old, old_lang_dir, language)?;
            let new_pack = side_pack(&new, new_lang_dir, language)?;
            let text = bundle_diff::diff_text(
                language,
                Labels::new(&old, old_pack.as_ref()),
                Labels::new(&new, new_pack.as_ref()),
            );
            output.push_str(&text.report());
        }
    }
    print!("{output}");
    Ok(())
}

fn side_pack(
    dataset: &Dataset,
    lang_dir: Option<&Path>,
    language: GameLanguage,
) -> Result<Option<ValidatedLanguagePack>, AppError> {
    match lang_dir {
        Some(directory) => import::load_paired_pack(dataset, directory, language),
        None => Ok(None),
    }
}

fn run_reindex(
    data_file: &Path,
    index_file: &Path,
    lang_dir: &Path,
    force: bool,
) -> Result<(), AppError> {
    let dataset = storage::load_dataset(data_file)?;
    let index = FilterEngine::build_stored_index(&dataset);
    index.validate(&dataset)?;
    index_storage::write_index(index_file, &index, force)?;
    let packs = import::reindex_language_packs(&dataset, lang_dir)?;
    println!(
        "Indexed {} locations into {} and rebuilt {} language packs in {}",
        dataset.stored.locations.len(),
        index_file.display(),
        packs.len(),
        lang_dir.display()
    );
    Ok(())
}

fn run_import(
    data_file: &Path,
    index_file: &Path,
    game_dir: Option<&Path>,
    lang_dir: &Path,
    languages: &[GameLanguage],
    force: bool,
) -> Result<(), AppError> {
    let installation = steam::discover(game_dir)?;
    let stored = import_game(&installation, print_progress)?;
    storage::write_dataset(data_file, &stored, force)?;
    // Packs pair with the bundle exactly as it reads back from disk.
    let dataset = storage::load_dataset(data_file)?;
    let index = FilterEngine::build_stored_index(&dataset);
    index.validate(&dataset)?;
    index_storage::write_index(index_file, &index, force)?;
    let packs = import::write_language_packs(
        &installation,
        &dataset,
        languages,
        lang_dir,
        force,
        print_progress,
    )?;
    println!(
        "Imported {} locations from EU5 build {} into {} and {}, with {} language packs in {}",
        dataset.stored.locations.len(),
        dataset.stored.build_id,
        data_file.display(),
        index_file.display(),
        packs.len(),
        lang_dir.display()
    );
    Ok(())
}

fn run_fonts(
    source_dir: PathBuf,
    output_dir: PathBuf,
    catalog_dir: PathBuf,
    lang_dir: PathBuf,
) -> Result<(), AppError> {
    let args = FontPackArgs {
        source_dir,
        output_dir,
        catalog_dir,
        lang_dir,
    };
    // The report's shape belongs to the font-pack module; success is reported by path only.
    let _report = font_pack::run(&args)?;
    println!("Wrote font packs into {}", args.output_dir.display());
    Ok(())
}
