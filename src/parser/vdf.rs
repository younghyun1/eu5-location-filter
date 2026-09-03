//! Minimal Valve Data Format parser used for Steam metadata.

use crate::AppError;

/// One VDF key/value pair.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VdfEntry {
    /// Entry key.
    pub key: String,
    /// Scalar or object value.
    pub value: VdfValue,
}

/// Supported VDF value types.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VdfValue {
    /// String scalar.
    Atom(String),
    /// Nested object.
    Object(Vec<VdfEntry>),
}

/// Parses the quoted subset emitted by Steam's VDF files.
pub fn parse_vdf(source_name: &str, input: &[u8]) -> Result<Vec<VdfEntry>, AppError> {
    let tokens = tokenize(source_name, input)?;
    let mut position = 0;
    let result = parse_entries(source_name, &tokens, &mut position, false)?;
    if position != tokens.len() {
        return Err(AppError::parse(
            source_name,
            position,
            "unexpected trailing VDF token",
        ));
    }
    Ok(result)
}

#[derive(Clone, Debug)]
enum Token {
    Text(String),
    LeftBrace,
    RightBrace,
}

fn parse_entries(
    source_name: &str,
    tokens: &[Token],
    position: &mut usize,
    in_object: bool,
) -> Result<Vec<VdfEntry>, AppError> {
    let mut entries = Vec::new();
    loop {
        let Some(token) = tokens.get(*position) else {
            if in_object {
                return Err(AppError::parse(
                    source_name,
                    *position,
                    "unterminated VDF object",
                ));
            }
            return Ok(entries);
        };
        if matches!(token, Token::RightBrace) {
            if !in_object {
                return Err(AppError::parse(
                    source_name,
                    *position,
                    "unmatched VDF brace",
                ));
            }
            *position += 1;
            return Ok(entries);
        }
        let Token::Text(key) = token else {
            return Err(AppError::parse(source_name, *position, "expected VDF key"));
        };
        let key = key.clone();
        *position += 1;
        let Some(value_token) = tokens.get(*position) else {
            return Err(AppError::parse(source_name, *position, "missing VDF value"));
        };
        let value = match value_token {
            Token::Text(value) => {
                *position += 1;
                VdfValue::Atom(value.clone())
            }
            Token::LeftBrace => {
                *position += 1;
                VdfValue::Object(parse_entries(source_name, tokens, position, true)?)
            }
            Token::RightBrace => {
                return Err(AppError::parse(source_name, *position, "invalid VDF value"));
            }
        };
        entries.push(VdfEntry { key, value });
    }
}

fn tokenize(source_name: &str, input: &[u8]) -> Result<Vec<Token>, AppError> {
    let mut result = Vec::new();
    let mut offset = 0;
    while offset < input.len() {
        match input[offset] {
            byte if byte.is_ascii_whitespace() => offset += 1,
            b'/' if input.get(offset + 1) == Some(&b'/') => {
                while offset < input.len() && input[offset] != b'\n' {
                    offset += 1;
                }
            }
            b'{' => {
                result.push(Token::LeftBrace);
                offset += 1;
            }
            b'}' => {
                result.push(Token::RightBrace);
                offset += 1;
            }
            b'"' => result.push(Token::Text(read_string(source_name, input, &mut offset)?)),
            _ => result.push(Token::Text(read_bare(source_name, input, &mut offset)?)),
        }
    }
    Ok(result)
}

fn read_string(source_name: &str, input: &[u8], offset: &mut usize) -> Result<String, AppError> {
    let start = *offset;
    *offset += 1;
    let mut output = Vec::new();
    while *offset < input.len() {
        let byte = input[*offset];
        *offset += 1;
        if byte == b'"' {
            return String::from_utf8(output).map_err(|error| {
                AppError::parse(source_name, start, format!("invalid UTF-8: {error}"))
            });
        }
        if byte == b'\\' {
            let Some(escaped) = input.get(*offset) else {
                return Err(AppError::parse(
                    source_name,
                    start,
                    "unterminated VDF escape",
                ));
            };
            *offset += 1;
            output.push(*escaped);
        } else {
            output.push(byte);
        }
    }
    Err(AppError::parse(
        source_name,
        start,
        "unterminated VDF string",
    ))
}

fn read_bare(source_name: &str, input: &[u8], offset: &mut usize) -> Result<String, AppError> {
    let start = *offset;
    while *offset < input.len()
        && !input[*offset].is_ascii_whitespace()
        && !matches!(input[*offset], b'{' | b'}')
    {
        *offset += 1;
    }
    std::str::from_utf8(&input[start..*offset])
        .map(str::to_owned)
        .map_err(|error| AppError::parse(source_name, start, format!("invalid UTF-8: {error}")))
}

#[cfg(test)]
mod tests {
    use super::{VdfValue, parse_vdf};

    #[test]
    fn parses_nested_steam_data() {
        let parsed = parse_vdf("test", b"\"root\" { \"path\" \"C:\\\\Steam\" // x\n }");
        assert!(parsed.is_ok());
        let Ok(entries) = parsed else { return };
        assert!(matches!(
            entries.first().map(|entry| &entry.value),
            Some(VdfValue::Object(_))
        ));
    }
}
