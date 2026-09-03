//! Parser for Paradox assignment files.

use crate::AppError;

/// One item inside a Paradox document or block.
#[derive(Clone, Debug, PartialEq)]
pub enum Entry {
    /// `key = value`.
    Assignment(String, Value),
    /// Bare value inside a list block.
    Atom(String),
}

/// A scalar or nested block.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// Unquoted or quoted scalar.
    Atom(String),
    /// Ordered block contents.
    Block(Vec<Entry>),
}

#[derive(Clone, Debug)]
enum TokenKind {
    Word(String),
    Equals,
    LeftBrace,
    RightBrace,
}

#[derive(Clone, Debug)]
struct Token {
    kind: TokenKind,
    offset: usize,
}

/// Parses a complete Paradox assignment document while retaining source order.
pub fn parse_document(source_name: &str, input: &[u8]) -> Result<Vec<Entry>, AppError> {
    let tokens = lex(source_name, input)?;
    let mut parser = Parser {
        source_name,
        tokens,
        position: 0,
    };
    let entries = parser.parse_entries(false)?;
    if parser.position != parser.tokens.len() {
        return Err(parser.error_here("unexpected trailing token"));
    }
    Ok(entries)
}

struct Parser<'a> {
    source_name: &'a str,
    tokens: Vec<Token>,
    position: usize,
}

impl Parser<'_> {
    fn parse_entries(&mut self, in_block: bool) -> Result<Vec<Entry>, AppError> {
        let mut entries = Vec::new();
        loop {
            let Some(token) = self.tokens.get(self.position) else {
                if in_block {
                    return Err(self.error_here("unterminated block"));
                }
                return Ok(entries);
            };
            if matches!(token.kind, TokenKind::RightBrace) {
                if !in_block {
                    return Err(self.error_here("unmatched closing brace"));
                }
                self.position += 1;
                return Ok(entries);
            }
            let TokenKind::Word(word) = &token.kind else {
                return Err(self.error_here("expected a key or list value"));
            };
            let word = word.clone();
            self.position += 1;
            if matches!(
                self.tokens.get(self.position).map(|item| &item.kind),
                Some(TokenKind::Equals)
            ) {
                self.position += 1;
                entries.push(Entry::Assignment(word, self.parse_value()?));
            } else if in_block {
                entries.push(Entry::Atom(word));
            } else {
                return Err(self.error_here("top-level values must be assignments"));
            }
        }
    }

    fn parse_value(&mut self) -> Result<Value, AppError> {
        let Some(token) = self.tokens.get(self.position) else {
            return Err(self.error_here("missing assignment value"));
        };
        match &token.kind {
            TokenKind::Word(value) => {
                let value = value.clone();
                self.position += 1;
                if matches!(
                    self.tokens.get(self.position).map(|item| &item.kind),
                    Some(TokenKind::LeftBrace)
                ) {
                    // Jomini uses tagged blocks such as `rgb { 1 2 3 }`. Callers
                    // interested only in assignment structure can treat these as blocks.
                    self.position += 1;
                    self.parse_entries(true).map(Value::Block)
                } else {
                    Ok(Value::Atom(value))
                }
            }
            TokenKind::LeftBrace => {
                self.position += 1;
                self.parse_entries(true).map(Value::Block)
            }
            TokenKind::Equals | TokenKind::RightBrace => {
                Err(self.error_here("invalid assignment value"))
            }
        }
    }

    fn error_here(&self, message: &str) -> AppError {
        let offset = self.tokens.get(self.position).map_or_else(
            || self.tokens.last().map_or(0, |token| token.offset),
            |token| token.offset,
        );
        AppError::parse(self.source_name, offset, message)
    }
}

fn lex(source_name: &str, input: &[u8]) -> Result<Vec<Token>, AppError> {
    let mut tokens = Vec::new();
    let mut offset = usize::from(input.starts_with(&[0xef, 0xbb, 0xbf])) * 3;
    while offset < input.len() {
        match input[offset] {
            byte if byte.is_ascii_whitespace() => offset += 1,
            b'#' => {
                while offset < input.len() && input[offset] != b'\n' {
                    offset += 1;
                }
            }
            b'=' => push_simple(&mut tokens, TokenKind::Equals, &mut offset),
            b'{' => push_simple(&mut tokens, TokenKind::LeftBrace, &mut offset),
            b'}' => push_simple(&mut tokens, TokenKind::RightBrace, &mut offset),
            b'"' => tokens.push(read_quoted(source_name, input, &mut offset)?),
            _ => tokens.push(read_word(source_name, input, &mut offset)?),
        }
    }
    Ok(tokens)
}

fn push_simple(tokens: &mut Vec<Token>, kind: TokenKind, offset: &mut usize) {
    tokens.push(Token {
        kind,
        offset: *offset,
    });
    *offset += 1;
}

fn read_word(source_name: &str, input: &[u8], offset: &mut usize) -> Result<Token, AppError> {
    let start = *offset;
    while *offset < input.len()
        && !input[*offset].is_ascii_whitespace()
        && !matches!(input[*offset], b'#' | b'=' | b'{' | b'}' | b'"')
    {
        *offset += 1;
    }
    if start == *offset {
        return Err(AppError::parse(source_name, start, "empty token"));
    }
    let value = std::str::from_utf8(&input[start..*offset]).map_err(|error| {
        AppError::parse(source_name, start, format!("invalid UTF-8 token: {error}"))
    })?;
    Ok(Token {
        kind: TokenKind::Word(value.to_owned()),
        offset: start,
    })
}

fn read_quoted(source_name: &str, input: &[u8], offset: &mut usize) -> Result<Token, AppError> {
    let start = *offset;
    *offset += 1;
    let mut value = Vec::new();
    let mut escaped = false;
    while *offset < input.len() {
        let byte = input[*offset];
        *offset += 1;
        if escaped {
            value.push(match byte {
                b'n' => b'\n',
                b'r' => b'\r',
                b't' => b'\t',
                other => other,
            });
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == b'"' {
            let value = String::from_utf8(value).map_err(|error| {
                AppError::parse(source_name, start, format!("invalid UTF-8 string: {error}"))
            })?;
            return Ok(Token {
                kind: TokenKind::Word(value),
                offset: start,
            });
        } else {
            value.push(byte);
        }
    }
    Err(AppError::parse(
        source_name,
        start,
        "unterminated quoted string",
    ))
}

#[cfg(test)]
mod tests {
    use super::{Entry, Value, parse_document};

    #[test]
    fn parses_comments_compact_blocks_and_lists() {
        let input = b"# heading\nloc={topography=flatland movement={-1.5 2} debug=rgb {1 2 3} name=\"A \\\"B\\\"\"}";
        let parsed = parse_document("test", input);
        assert!(parsed.is_ok());
        let Ok(entries) = parsed else { return };
        assert_eq!(entries.len(), 1);
        assert!(matches!(
            entries.first(),
            Some(Entry::Assignment(_, Value::Block(_)))
        ));
    }

    #[test]
    fn rejects_malformed_documents() {
        assert!(parse_document("test", b"a = { b = 1").is_err());
        assert!(parse_document("test", b"bare").is_err());
    }
}
