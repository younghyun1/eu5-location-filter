//! Line parsing for catalog files, following the rules in `assets/i18n/README.md`.

use std::collections::HashMap;

use super::model::{Catalog, Category, Form, Item, Value};

/// One successfully parsed entry line.
struct Parsed {
    key: String,
    category: Option<Category>,
    form: Form,
}

/// Parses one file, decoding escapes and checking keys and braces. Every line problem is
/// collected so one build reports them all.
pub fn parse_catalog(
    tag: &'static str,
    path: String,
    text: &str,
) -> Result<Catalog, Vec<String>> {
    let mut errors = Vec::new();
    let body = match text.strip_prefix('\u{feff}') {
        Some(rest) => {
            errors.push(format!("{path}:1: the file starts with a byte order mark"));
            rest
        }
        None => text,
    };
    let mut items: Vec<Item> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    for (number, raw) in body.lines().enumerate() {
        let line = number + 1;
        match parse_line(raw, line) {
            Ok(None) => {}
            Ok(Some(parsed)) => {
                if let Err(message) = insert(&mut items, &mut index, parsed, line) {
                    errors.push(format!("{path}:{line}: {message}"));
                }
            }
            Err(message) => errors.push(format!("{path}:{line}: {message}")),
        }
    }
    if errors.is_empty() {
        Ok(Catalog::new(tag, path, items))
    } else {
        Err(errors)
    }
}

/// Adds an entry to its base key, rejecting duplicates and plain/plural mixes.
fn insert(
    items: &mut Vec<Item>,
    index: &mut HashMap<String, usize>,
    parsed: Parsed,
    line: usize,
) -> Result<(), String> {
    let Parsed {
        key,
        category,
        form,
    } = parsed;
    let Some(position) = index.get(&key).copied() else {
        let value = match category {
            Some(category) => Value::Plural(vec![(category, form)]),
            None => Value::Plain(form),
        };
        index.insert(key.clone(), items.len());
        items.push(Item { key, line, value });
        return Ok(());
    };
    let Some(existing) = items.get_mut(position) else {
        return Err(format!("internal error: key `{key}` has no item"));
    };
    match (&mut existing.value, category) {
        (Value::Plain(_), None) => Err(format!("key `{key}` is defined twice")),
        (Value::Plural(forms), Some(category)) => {
            if forms.iter().any(|(known, _)| *known == category) {
                Err(format!(
                    "plural form [{}] of `{key}` is defined twice",
                    category.name()
                ))
            } else {
                forms.push((category, form));
                Ok(())
            }
        }
        _ => Err(format!(
            "key `{key}` is used both with and without plural forms"
        )),
    }
}

/// Parses one line; `Ok(None)` for blank and comment lines.
fn parse_line(raw: &str, line: usize) -> Result<Option<Parsed>, String> {
    let trimmed = raw.trim_matches([' ', '\t']);
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return Ok(None);
    }
    let Some((key_part, value_part)) = trimmed.split_once('=') else {
        return Err("expected an entry of the form `key = value`".to_owned());
    };
    let key_part = key_part.trim_matches([' ', '\t']);
    let value_part = value_part.trim_matches([' ', '\t']);
    if value_part.is_empty() {
        return Err(format!("key `{key_part}` has an empty value"));
    }
    let (key, category) = split_key(key_part)?;
    let decoded = decode_escapes(value_part)?;
    let placeholders = scan_braces(&decoded)?;
    let text = if placeholders.is_empty() {
        decoded.replace("{{", "{").replace("}}", "}")
    } else {
        decoded
    };
    Ok(Some(Parsed {
        key,
        category,
        form: Form {
            text,
            line,
            placeholders,
        },
    }))
}

/// Splits `key[category]` and validates the base key.
fn split_key(text: &str) -> Result<(String, Option<Category>), String> {
    let (base, category) = match text.strip_suffix(']') {
        None => (text, None),
        Some(head) => {
            let Some((base, name)) = head.split_once('[') else {
                return Err(format!("key `{text}` has `]` without `[`"));
            };
            match Category::parse(name) {
                Some(category) => (base, Some(category)),
                None => {
                    return Err(format!(
                        "unknown plural category `{name}`; use zero, one, two, few, many or other"
                    ));
                }
            }
        }
    };
    validate_key(base)?;
    Ok((base.to_owned(), category))
}

/// Keys are lower-case dotted snake case; each segment starts with a letter.
fn validate_key(key: &str) -> Result<(), String> {
    let invalid = || {
        format!(
            "invalid key `{key}`: use dot-separated segments of a-z, 0-9 and _, each starting with a letter"
        )
    };
    if key.is_empty() {
        return Err("empty key".to_owned());
    }
    for segment in key.split('.') {
        let mut characters = segment.chars();
        match characters.next() {
            Some(first) if first.is_ascii_lowercase() => {}
            _ => return Err(invalid()),
        }
        if !characters.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
            return Err(invalid());
        }
    }
    Ok(())
}

/// Decodes `\n`, `\s` and `\\`; any other backslash sequence is an error.
fn decode_escapes(raw: &str) -> Result<String, String> {
    let mut out = String::with_capacity(raw.len());
    let mut characters = raw.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            out.push(character);
            continue;
        }
        match characters.next() {
            Some('n') => out.push('\n'),
            Some('s') => out.push(' '),
            Some('\\') => out.push('\\'),
            Some(other) => {
                return Err(format!(
                    "unknown escape `\\{other}`; use \\n, \\s or \\\\"
                ));
            }
            None => return Err("the value ends with a lone backslash".to_owned()),
        }
    }
    Ok(out)
}

/// Placeholder names must use a-z, 0-9 and _ and start with a letter.
fn valid_name(name: &str) -> bool {
    let mut characters = name.chars();
    matches!(characters.next(), Some(first) if first.is_ascii_lowercase())
        && characters.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// Returns the placeholder names of a decoded value, rejecting braces that are neither a
/// placeholder nor a doubled `{{` or `}}`.
fn scan_braces(text: &str) -> Result<Vec<String>, String> {
    let mut names = Vec::new();
    let mut characters = text.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            '{' => {
                if characters.peek() == Some(&'{') {
                    characters.next();
                    continue;
                }
                let mut name = String::new();
                let mut closed = false;
                for next in characters.by_ref() {
                    if next == '}' {
                        closed = true;
                        break;
                    }
                    name.push(next);
                }
                if !closed {
                    return Err("`{` without a closing `}`; write `{{` for a literal brace".to_owned());
                }
                if !valid_name(&name) {
                    return Err(format!(
                        "`{{{name}}}` is not a placeholder; names use a-z, 0-9 and _ and start with a letter"
                    ));
                }
                names.push(name);
            }
            '}' => {
                if characters.peek() == Some(&'}') {
                    characters.next();
                } else {
                    return Err("single `}`; write `}}` for a literal brace".to_owned());
                }
            }
            _ => {}
        }
    }
    Ok(names)
}
