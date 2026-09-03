//! Line-oriented Paradox localization parsing.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::AppError;

const MAX_LOCALIZATION_FILE_SIZE: u64 = 32 * 1024 * 1024;
const MAX_LOCALIZATION_FILES: usize = 10_000;
const MAX_LOCALIZATION_TOTAL_SIZE: u64 = 512 * 1024 * 1024;
const MAX_LOCALIZATION_ENTRIES: usize = 2_000_000;
const MAX_LOCALIZATION_TEXT: usize = 256 * 1024 * 1024;
const MAX_REFERENCE_DEPTH: usize = 32;

/// Parses `key:0 "Value"` and returns `None` for headings, comments, and blank lines.
pub fn parse_localization_line(line: &str) -> Result<Option<(String, String)>, AppError> {
    let line = line.trim_start_matches('\u{feff}').trim();
    if line.is_empty() || line.starts_with('#') || line.ends_with(':') && !line.contains('"') {
        return Ok(None);
    }
    let Some(colon) = line.find(':') else {
        return Ok(None);
    };
    let key = line[..colon].trim();
    if key.is_empty() {
        return Err(AppError::parse(
            "localization",
            colon,
            "empty localization key",
        ));
    }
    let mut remainder = line[colon + 1..].trim_start();
    remainder = remainder.trim_start_matches(|character: char| character.is_ascii_digit());
    remainder = remainder.trim_start();
    if remainder.is_empty() || remainder.starts_with('#') {
        return Ok(None);
    }
    if !remainder.starts_with('"') {
        return Err(AppError::parse(
            "localization",
            colon + 1,
            "missing quoted localization value",
        ));
    }
    let (value, _) = parse_quoted(remainder)?;
    Ok(Some((key.to_owned(), value)))
}

/// Recursively resolves requested English localization keys in deterministic path order.
pub fn read_localizations(
    roots: &[PathBuf],
    requested: &HashSet<String>,
) -> Result<HashMap<String, String>, AppError> {
    let mut paths = Vec::new();
    for root in roots {
        collect_files(root, &mut paths)?;
    }
    paths.sort();
    if paths.len() > MAX_LOCALIZATION_FILES {
        return Err(AppError::InvalidData(
            "localization file count exceeds configured limit".to_owned(),
        ));
    }
    let mut raw_values = HashMap::new();
    let mut total_source_size = 0_u64;
    let mut total_text_size = 0_usize;
    for path in paths {
        let metadata =
            fs::metadata(&path).map_err(|source| AppError::io("inspect", &path, source))?;
        if metadata.len() > MAX_LOCALIZATION_FILE_SIZE {
            return Err(AppError::InvalidData(format!(
                "localization file is too large: {}",
                path.display()
            )));
        }
        total_source_size = total_source_size.saturating_add(metadata.len());
        if total_source_size > MAX_LOCALIZATION_TOTAL_SIZE {
            return Err(AppError::InvalidData(
                "English localization sources exceed the cumulative size limit".to_owned(),
            ));
        }
        let bytes = fs::read(&path).map_err(|source| AppError::io("read", &path, source))?;
        let text = String::from_utf8(bytes).map_err(|error| {
            AppError::parse(
                path.display().to_string(),
                0,
                format!("invalid UTF-8: {error}"),
            )
        })?;
        for (line_number, line) in text.lines().enumerate() {
            let candidate = line.trim_start_matches('\u{feff}').trim_start();
            let Some(colon) = candidate.find(':') else {
                continue;
            };
            if candidate[..colon].trim().is_empty() {
                continue;
            }
            match parse_localization_line(line) {
                Ok(Some((key, value))) => {
                    total_text_size = total_text_size.saturating_add(key.len() + value.len());
                    if raw_values.len() >= MAX_LOCALIZATION_ENTRIES
                        || total_text_size > MAX_LOCALIZATION_TEXT
                    {
                        return Err(AppError::InvalidData(
                            "English localization index exceeds configured limits".to_owned(),
                        ));
                    }
                    raw_values.insert(key, value);
                }
                Ok(_) => {}
                Err(error) => {
                    return Err(AppError::parse(
                        path.display().to_string(),
                        line_number,
                        error.to_string(),
                    ));
                }
            }
        }
    }
    let mut cache = HashMap::with_capacity(requested.len());
    for key in requested {
        let mut visiting = HashSet::new();
        if let Some(value) = resolve_key(key, &raw_values, &mut cache, &mut visiting, 0) {
            cache.insert(key.clone(), value);
        }
    }
    cache.retain(|key, _| requested.contains(key));
    Ok(cache)
}

fn resolve_key(
    key: &str,
    raw: &HashMap<String, String>,
    cache: &mut HashMap<String, String>,
    visiting: &mut HashSet<String>,
    depth: usize,
) -> Option<String> {
    if let Some(value) = cache.get(key) {
        return Some(value.clone());
    }
    let value = raw.get(key)?;
    if depth >= MAX_REFERENCE_DEPTH || !visiting.insert(key.to_owned()) {
        return Some(value.clone());
    }
    let expanded = expand_references(value, raw, cache, visiting, depth + 1);
    visiting.remove(key);
    cache.insert(key.to_owned(), expanded.clone());
    Some(expanded)
}

fn expand_references(
    value: &str,
    raw: &HashMap<String, String>,
    cache: &mut HashMap<String, String>,
    visiting: &mut HashSet<String>,
    depth: usize,
) -> String {
    let mut output = String::with_capacity(value.len());
    let mut remainder = value;
    while let Some(start) = remainder.find('$') {
        output.push_str(&remainder[..start]);
        let after_start = &remainder[start + 1..];
        let Some(end) = after_start.find('$') else {
            output.push_str(&remainder[start..]);
            return output;
        };
        let token = &after_start[..end];
        let key = token.split('|').next().unwrap_or_default();
        if let Some(resolved) = resolve_key(key, raw, cache, visiting, depth) {
            output.push_str(&resolved);
        } else {
            output.push('$');
            output.push_str(token);
            output.push('$');
        }
        remainder = &after_start[end + 1..];
    }
    output.push_str(remainder);
    output
}

fn collect_files(directory: &Path, output: &mut Vec<PathBuf>) -> Result<(), AppError> {
    let entries =
        fs::read_dir(directory).map_err(|source| AppError::io("list", directory, source))?;
    for entry in entries {
        let entry = entry.map_err(|source| AppError::io("list", directory, source))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|source| AppError::io("inspect", &path, source))?;
        if file_type.is_dir() {
            collect_files(&path, output)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("yml"))
        {
            output.push(path);
        }
    }
    Ok(())
}

fn parse_quoted(input: &str) -> Result<(String, usize), AppError> {
    let mut output = String::new();
    let mut escaped = false;
    for (offset, character) in input[1..].char_indices() {
        if escaped {
            output.push(match character {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                other => other,
            });
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == '"' {
            return Ok((output, offset + 2));
        } else {
            output.push(character);
        }
    }
    Err(AppError::parse(
        "localization",
        0,
        "unterminated localization value",
    ))
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::{parse_localization_line, resolve_key};

    #[test]
    fn parses_escaped_localization() {
        let parsed = parse_localization_line("\u{feff}stockholm:0 \"Stock\\\"holm\" # comment");
        assert!(parsed.is_ok());
        assert_eq!(
            parsed.ok().flatten(),
            Some(("stockholm".to_owned(), "Stock\"holm".to_owned()))
        );
    }

    #[test]
    fn ignores_headers_and_comments() {
        assert_eq!(parse_localization_line("l_english:").ok().flatten(), None);
        assert_eq!(
            parse_localization_line("l_english: # Ukrainian")
                .ok()
                .flatten(),
            None
        );
        assert_eq!(parse_localization_line(" # text").ok().flatten(), None);
    }

    #[test]
    fn resolves_nested_static_references() {
        let raw = HashMap::from([
            ("place".to_owned(), "$short$ of Lewis".to_owned()),
            ("short".to_owned(), "Butt".to_owned()),
        ]);
        let resolved = resolve_key("place", &raw, &mut HashMap::new(), &mut HashSet::new(), 0);
        assert_eq!(resolved.as_deref(), Some("Butt of Lewis"));
    }
}
