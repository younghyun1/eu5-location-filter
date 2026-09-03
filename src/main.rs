#![cfg_attr(windows, windows_subsystem = "windows")]

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use eu5_location_filter::filter::FilterEngine;
use eu5_location_filter::import::import_game;
use eu5_location_filter::{index_storage, steam, storage, ui};

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
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Import a vanilla EU5 installation without opening the GUI.
    Import {
        /// Replace an existing valid data file.
        #[arg(long)]
        force: bool,
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
    let result = match cli.command {
        Some(Command::Import { force }) => run_import(
            cli.data_file.as_deref(),
            cli.index_file.as_deref(),
            cli.game_dir.as_deref(),
            force,
        ),
        None => ui::run(cli.data_file, cli.index_file, cli.game_dir),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(1)
        }
    }
}

fn run_import(
    data_file: Option<&std::path::Path>,
    index_file: Option<&std::path::Path>,
    game_dir: Option<&std::path::Path>,
    force: bool,
) -> Result<(), eu5_location_filter::AppError> {
    let data_file =
        data_file.unwrap_or_else(|| std::path::Path::new("assets/eu5-locations.bitcode.zst"));
    let index_file =
        index_file.unwrap_or_else(|| std::path::Path::new("assets/eu5-indexes.bitcode.zst"));
    let installation = steam::discover(game_dir)?;
    let stored = import_game(&installation, |progress| {
        if progress.total > 0 {
            eprintln!(
                "{} ({}/{})",
                progress.stage, progress.current, progress.total
            );
        } else {
            eprintln!("{}", progress.stage);
        }
    })?;
    storage::write_dataset(data_file, &stored, force)?;
    let dataset = storage::load_dataset(data_file)?;
    let index = FilterEngine::build_stored_index(&dataset);
    index.validate(&dataset)?;
    index_storage::write_index(index_file, &index, force)?;
    println!(
        "Imported {} locations from EU5 build {} into {} and {}",
        stored.locations.len(),
        stored.build_id,
        data_file.display(),
        index_file.display()
    );
    Ok(())
}
