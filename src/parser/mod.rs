//! Bounded parsers for the small text formats used by Steam and Paradox.

mod localization;
mod paradox;
mod semicolon;
mod vdf;

pub use localization::{parse_localization_line, read_localizations};
pub use paradox::{Entry, Value, parse_document};
pub use semicolon::parse_semicolon_line;
pub use vdf::{VdfEntry, VdfValue, parse_vdf};

use std::fs;
use std::path::Path;

use crate::AppError;

/// Reads a source file only after enforcing a format-specific byte limit.
pub fn read_limited(path: &Path, limit: u64) -> Result<Vec<u8>, AppError> {
    let metadata = fs::metadata(path).map_err(|source| AppError::io("inspect", path, source))?;
    if metadata.len() > limit {
        return Err(AppError::InvalidData(format!(
            "{} is {} bytes; limit is {limit}",
            path.display(),
            metadata.len()
        )));
    }
    fs::read(path).map_err(|source| AppError::io("read", path, source))
}
