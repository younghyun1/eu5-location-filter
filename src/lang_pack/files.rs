//! Desktop pack files: bounded reads and crash-safe replacement.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use super::{MAX_PACK_COMPRESSED_SIZE, StoredLanguagePack, decode_pack, encode_pack};
use crate::AppError;

/// Reads a pack file after enforcing `MAX_PACK_COMPRESSED_SIZE` on its metadata.
pub fn load_pack(path: &Path) -> Result<StoredLanguagePack, AppError> {
    let size = fs::metadata(path)
        .map_err(|source| AppError::io("inspect", path, source))?
        .len();
    if size > MAX_PACK_COMPRESSED_SIZE {
        return Err(AppError::InvalidData(format!(
            "language pack file exceeds its size limit: {}",
            path.display()
        )));
    }
    let bytes = fs::read(path).map_err(|source| AppError::io("read", path, source))?;
    decode_pack(&bytes)
}

/// Writes through a verified temporary file, then removes same-language packs of other builds
/// so the build script finds exactly one file per language.
pub fn write_pack(path: &Path, pack: &StoredLanguagePack, force: bool) -> Result<(), AppError> {
    if path.exists() && !force {
        return Err(AppError::InvalidData(format!(
            "language pack already exists; pass --force to replace it: {}",
            path.display()
        )));
    }
    let parent = path
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|source| AppError::io("create language pack directory", parent, source))?;
    let bytes = encode_pack(pack)?;
    let temp = suffixed(path, &format!(".{}.tmp", std::process::id()));
    remove_if_exists(&temp)?;
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temp)
        .map_err(|source| AppError::io("create temporary language pack", &temp, source))?;
    file.write_all(&bytes)
        .and_then(|()| file.flush())
        .and_then(|()| file.sync_all())
        .map_err(|source| AppError::io("write temporary language pack", &temp, source))?;
    drop(file);
    // Read back what reached the disk, so a truncated or corrupt write never replaces a pack.
    match load_pack(&temp) {
        Ok(decoded) if decoded == *pack => {}
        Ok(_) => {
            remove_if_exists(&temp)?;
            return Err(AppError::InvalidData(
                "written language pack does not round-trip".to_owned(),
            ));
        }
        Err(error) => {
            remove_if_exists(&temp)?;
            return Err(error);
        }
    }
    replace(path, &temp)?;
    remove_other_builds(parent, pack)
}

fn replace(path: &Path, temp: &Path) -> Result<(), AppError> {
    let backup = suffixed(path, ".bak");
    remove_if_exists(&backup)?;
    if path.exists() {
        fs::rename(path, &backup)
            .map_err(|source| AppError::io("back up language pack", path, source))?;
    }
    if let Err(source) = fs::rename(temp, path) {
        if backup.exists() {
            // Best effort: the install error below is the one worth reporting.
            let _ = fs::rename(&backup, path);
        }
        return Err(AppError::io("install language pack", path, source));
    }
    remove_if_exists(&backup)
}

/// Removes `<code>.<digits>.bitcode.zst` files of this language whose build differs.
///
/// Requiring digits between the code and the extension leaves other languages and the
/// temporary and backup files of this write alone.
fn remove_other_builds(directory: &Path, pack: &StoredLanguagePack) -> Result<(), AppError> {
    let entries = fs::read_dir(directory)
        .map_err(|source| AppError::io("list language packs", directory, source))?;
    let prefix = format!("{}.", pack.language.code());
    for entry in entries {
        let entry =
            entry.map_err(|source| AppError::io("list language packs", directory, source))?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        let build = name
            .strip_prefix(prefix.as_str())
            .and_then(|rest| rest.strip_suffix(".bitcode.zst"))
            .filter(|digits| !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()))
            .and_then(|digits| digits.parse::<u64>().ok());
        if build.is_some_and(|build| build != pack.build_id) {
            remove_if_exists(&entry.path())?;
        }
    }
    Ok(())
}

fn suffixed(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path
        .file_name()
        .map_or_else(|| "language-pack".into(), |value| value.to_os_string());
    name.push(suffix);
    path.with_file_name(name)
}

fn remove_if_exists(path: &Path) -> Result<(), AppError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(AppError::io("remove stale language pack", path, source)),
    }
}
