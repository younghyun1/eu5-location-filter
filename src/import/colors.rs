//! Named map-color extraction.

use std::collections::HashMap;

use crate::AppError;
use crate::model::MapColor;
use crate::parser::{Entry, Value, parse_document};

pub(super) fn parse_colors(
    source_name: &str,
    input: &[u8],
) -> Result<HashMap<String, MapColor>, AppError> {
    let entries = parse_document(source_name, input)?;
    let mut by_key = HashMap::with_capacity(entries.len());
    let mut by_color = HashMap::with_capacity(entries.len());
    for entry in entries {
        let Entry::Assignment(key, Value::Atom(raw_color)) = entry else {
            return Err(AppError::InvalidData(
                "named colors must be scalar assignments".to_owned(),
            ));
        };
        let color = MapColor::parse(&raw_color)?;
        if by_key.insert(key.clone(), color).is_some() {
            return Err(AppError::InvalidData(format!(
                "duplicate named color identifier: {key}"
            )));
        }
        if let Some(existing) = by_color.insert(color, key.clone()) {
            return Err(AppError::InvalidData(format!(
                "duplicate map color {} for {existing} and {key}",
                color.hex()
            )));
        }
    }
    Ok(by_key)
}

#[cfg(test)]
mod tests {
    use super::parse_colors;
    use crate::model::MapColor;

    #[test]
    fn accepts_short_rgb_and_rejects_duplicates() {
        let parsed = parse_colors("test", b"a = 1e05");
        assert_eq!(
            parsed.ok().and_then(|map| map.get("a").copied()),
            Some(MapColor(0x001e05))
        );
        assert!(parse_colors("test", b"a=1 b=000001").is_err());
    }
}
