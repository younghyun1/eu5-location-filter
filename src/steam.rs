//! Steam installation discovery and validation.

use std::path::{Path, PathBuf};

use crate::AppError;
use crate::model::EU5_APP_ID;
use crate::parser::{VdfEntry, VdfValue, parse_vdf, read_limited};

const MAX_STEAM_METADATA_SIZE: u64 = 4 * 1024 * 1024;
const REQUIRED_MAP_FILES: &[&str] = &[
    "location_templates.txt",
    "definitions.txt",
    "named_locations/00_default.txt",
    "ports.csv",
    "locations.png",
    "rivers.png",
    "default.map",
];

/// A validated vanilla game installation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameInstallation {
    /// Directory containing the installed executable and `game` directory.
    pub root: PathBuf,
    /// Build ID from Steam's app manifest, or zero for an explicit non-Steam path.
    pub build_id: u64,
}

impl GameInstallation {
    /// Returns the vanilla map-data directory.
    #[must_use]
    pub fn map_data(&self) -> PathBuf {
        self.root.join("game").join("in_game").join("map_data")
    }

    /// Returns English localization roots used by the loading and main-menu scopes.
    #[must_use]
    pub fn localization_roots(&self) -> Vec<PathBuf> {
        ["loading_screen", "main_menu"]
            .into_iter()
            .map(|scope| {
                self.root
                    .join("game")
                    .join(scope)
                    .join("localization")
                    .join("english")
            })
            .filter(|path| path.is_dir())
            .collect()
    }

    /// Returns the game override for the Jomini river defines.
    #[must_use]
    pub fn river_defines(&self) -> PathBuf {
        self.root
            .join("game")
            .join("loading_screen")
            .join("common")
            .join("defines")
            .join("jomini")
            .join("rivers.txt")
    }

    /// Returns one immutable map-data definition used by capacity calculation.
    #[must_use]
    pub fn map_definition(&self) -> PathBuf {
        self.map_data().join("default.map")
    }

    /// Returns the vanilla topography definitions.
    #[must_use]
    pub fn topography_definitions(&self) -> PathBuf {
        self.root
            .join("game/in_game/common/topography/00_default.txt")
    }

    /// Returns the vanilla vegetation definitions.
    #[must_use]
    pub fn vegetation_definitions(&self) -> PathBuf {
        self.root
            .join("game/in_game/common/vegetation/00_default.txt")
    }

    /// Returns the vanilla climate definitions.
    #[must_use]
    pub fn climate_definitions(&self) -> PathBuf {
        self.root
            .join("game/in_game/common/climates/00_default.txt")
    }

    /// Returns static location modifiers used by immutable map factors.
    #[must_use]
    pub fn location_static_modifiers(&self) -> PathBuf {
        self.root
            .join("game/main_menu/common/static_modifiers/location.txt")
    }
}

/// Uses an explicit path or discovers Steam on Windows, then validates required inputs.
pub fn discover(game_dir: Option<&Path>) -> Result<GameInstallation, AppError> {
    match game_dir {
        Some(path) => validate_explicit(path),
        None => discover_steam(),
    }
}

fn validate_explicit(path: &Path) -> Result<GameInstallation, AppError> {
    let build_id = manifest_for_game_root(path)
        .and_then(|manifest| read_manifest(&manifest).ok())
        .map_or(0, |(_, build_id)| build_id);
    validate(path, build_id)
}

fn validate(path: &Path, build_id: u64) -> Result<GameInstallation, AppError> {
    let root = path
        .canonicalize()
        .map_err(|source| AppError::io("resolve game directory", path, source))?;
    let map_data = root.join("game").join("in_game").join("map_data");
    for relative in REQUIRED_MAP_FILES {
        let candidate = map_data.join(relative);
        if !candidate.is_file() {
            return Err(AppError::MissingFile(candidate));
        }
    }
    let river_defines = root.join("game/loading_screen/common/defines/jomini/rivers.txt");
    if !river_defines.is_file() {
        return Err(AppError::MissingFile(river_defines));
    }
    for required in [
        root.join("game/in_game/common/topography/00_default.txt"),
        root.join("game/in_game/common/vegetation/00_default.txt"),
        root.join("game/in_game/common/climates/00_default.txt"),
        root.join("game/main_menu/common/static_modifiers/location.txt"),
    ] {
        if !required.is_file() {
            return Err(AppError::MissingFile(required));
        }
    }
    Ok(GameInstallation { root, build_id })
}

fn manifest_for_game_root(root: &Path) -> Option<PathBuf> {
    let common = root.parent()?;
    if !common
        .file_name()?
        .to_string_lossy()
        .eq_ignore_ascii_case("common")
    {
        return None;
    }
    Some(
        common
            .parent()?
            .join(format!("appmanifest_{EU5_APP_ID}.acf")),
    )
}

#[cfg(windows)]
fn discover_steam() -> Result<GameInstallation, AppError> {
    use std::process::Command;

    let output = Command::new("reg.exe")
        .args(["query", r"HKCU\Software\Valve\Steam", "/v", "SteamPath"])
        .output()
        .map_err(|source| AppError::io("query Steam registry key", "reg.exe", source))?;
    if !output.status.success() {
        return Err(AppError::Installation(
            "SteamPath registry value is unavailable".to_owned(),
        ));
    }
    let stdout = String::from_utf8(output.stdout).map_err(|error| {
        AppError::Installation(format!("Steam registry output is not UTF-8: {error}"))
    })?;
    let steam_path = stdout
        .lines()
        .find_map(|line| line.split_once("REG_SZ").map(|(_, value)| value.trim()))
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AppError::Installation("SteamPath registry value is empty".to_owned()))?;
    discover_in_steam(Path::new(steam_path))
}

#[cfg(not(windows))]
fn discover_steam() -> Result<GameInstallation, AppError> {
    Err(AppError::Installation(
        "automatic Steam discovery is supported only on Windows; pass --game-dir".to_owned(),
    ))
}

#[cfg(windows)]
fn discover_in_steam(steam_root: &Path) -> Result<GameInstallation, AppError> {
    let libraries_path = steam_root.join("steamapps").join("libraryfolders.vdf");
    let bytes = read_limited(&libraries_path, MAX_STEAM_METADATA_SIZE)?;
    let entries = parse_vdf(&libraries_path.display().to_string(), &bytes)?;
    let libraries = object_at(&entries, "libraryfolders").ok_or_else(|| {
        AppError::Installation("libraryfolders.vdf has no libraryfolders object".to_owned())
    })?;
    for entry in libraries {
        let VdfValue::Object(library) = &entry.value else {
            continue;
        };
        if object_at(library, "apps")
            .is_none_or(|apps| atom_at(apps, &EU5_APP_ID.to_string()).is_none())
        {
            continue;
        }
        let Some(path) = atom_at(library, "path") else {
            continue;
        };
        let manifest = Path::new(path)
            .join("steamapps")
            .join(format!("appmanifest_{EU5_APP_ID}.acf"));
        let (install_dir, build_id) = read_manifest(&manifest)?;
        return validate(
            &Path::new(path)
                .join("steamapps")
                .join("common")
                .join(install_dir),
            build_id,
        );
    }
    Err(AppError::Installation(format!(
        "Steam app {EU5_APP_ID} is not installed"
    )))
}

fn read_manifest(path: &Path) -> Result<(String, u64), AppError> {
    let bytes = read_limited(path, MAX_STEAM_METADATA_SIZE)?;
    let entries = parse_vdf(&path.display().to_string(), &bytes)?;
    let app = object_at(&entries, "AppState")
        .ok_or_else(|| AppError::Installation("app manifest has no AppState object".to_owned()))?;
    let install_dir = atom_at(app, "installdir")
        .ok_or_else(|| AppError::Installation("app manifest has no installdir".to_owned()))?;
    let build_id = atom_at(app, "buildid")
        .ok_or_else(|| AppError::Installation("app manifest has no buildid".to_owned()))?
        .parse::<u64>()
        .map_err(|error| AppError::Installation(format!("invalid Steam build ID: {error}")))?;
    Ok((install_dir.to_owned(), build_id))
}

fn object_at<'a>(entries: &'a [VdfEntry], key: &str) -> Option<&'a [VdfEntry]> {
    entries.iter().find_map(|entry| {
        if entry.key.eq_ignore_ascii_case(key) {
            match &entry.value {
                VdfValue::Object(value) => Some(value.as_slice()),
                VdfValue::Atom(_) => None,
            }
        } else {
            None
        }
    })
}

fn atom_at<'a>(entries: &'a [VdfEntry], key: &str) -> Option<&'a str> {
    entries.iter().find_map(|entry| {
        if entry.key.eq_ignore_ascii_case(key) {
            match &entry.value {
                VdfValue::Atom(value) => Some(value.as_str()),
                VdfValue::Object(_) => None,
            }
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{atom_at, object_at};
    use crate::parser::{VdfEntry, VdfValue};

    #[test]
    fn finds_values_case_insensitively() {
        let entries = vec![VdfEntry {
            key: "Root".to_owned(),
            value: VdfValue::Object(vec![VdfEntry {
                key: "BuildID".to_owned(),
                value: VdfValue::Atom("42".to_owned()),
            }]),
        }];
        let root = object_at(&entries, "root");
        assert!(root.is_some());
        assert_eq!(root.and_then(|value| atom_at(value, "buildid")), Some("42"));
    }
}
