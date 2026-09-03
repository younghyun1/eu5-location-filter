//! Small semicolon-delimited row parser.

use crate::AppError;

/// Parses one semicolon-delimited row with CSV-style quoted fields.
pub fn parse_semicolon_line(line: &str) -> Result<Vec<String>, AppError> {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut characters = line.chars().peekable();
    let mut quoted = false;
    while let Some(character) = characters.next() {
        if quoted {
            if character == '"' {
                if characters.peek() == Some(&'"') {
                    field.push('"');
                    characters.next();
                } else {
                    quoted = false;
                }
            } else {
                field.push(character);
            }
        } else if character == ';' {
            fields.push(std::mem::take(&mut field));
        } else if character == '"' && field.is_empty() {
            quoted = true;
        } else {
            field.push(character);
        }
    }
    if quoted {
        return Err(AppError::parse(
            "semicolon CSV",
            line.len(),
            "unterminated quoted field",
        ));
    }
    fields.push(field);
    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::parse_semicolon_line;

    #[test]
    fn parses_empty_and_quoted_fields() {
        assert_eq!(
            parse_semicolon_line("a;\"b;b\";;").ok(),
            Some(vec![
                "a".to_owned(),
                "b;b".to_owned(),
                String::new(),
                String::new()
            ])
        );
    }
}
