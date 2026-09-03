use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use eu5_location_filter::import::import_game;
use eu5_location_filter::{steam, storage, ui};

#[derive(Debug, Parser)]
#[command(
    name = "eu5-location-filter",
    version = env!("CARGO_PKG_VERSION"),
    about = "Inspect and filter Europa Universalis V map locations"
)]
struct Cli {
    #[arg(long, global = true, default_value = "eu5-locations.bitcode.zst")]
    data_file: PathBuf,
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
        Some(Command::Import { force }) => {
            run_import(&cli.data_file, cli.game_dir.as_deref(), force)
        }
        None => ui::run(cli.data_file, cli.game_dir),
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
    data_file: &std::path::Path,
    game_dir: Option<&std::path::Path>,
    force: bool,
) -> Result<(), eu5_location_filter::AppError> {
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
    println!(
        "Imported {} locations from EU5 build {} into {}",
        stored.locations.len(),
        stored.build_id,
        data_file.display()
    );
    Ok(())
}
