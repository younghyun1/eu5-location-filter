//! Jomini river-width metadata extraction.

use crate::AppError;
use crate::model::RiverWidthMetadata;
use crate::parser::{Entry, Value, parse_document};

pub(super) fn parse_river_widths(
    source_name: &str,
    input: &[u8],
) -> Result<RiverWidthMetadata, AppError> {
    let entries = parse_document(source_name, input)?;
    let block = entries.into_iter().find_map(|entry| match entry {
        Entry::Assignment(key, Value::Block(values)) if key == "NRivers" => Some(values),
        _ => None,
    });
    let Some(block) = block else {
        return Err(AppError::InvalidData(
            "river defines have no NRivers block".to_owned(),
        ));
    };
    let level_count = scalar(&block, "NUM_WIDTH_PIXEL_VALUES")?
        .parse::<u8>()
        .map_err(|error| AppError::InvalidData(format!("invalid river width count: {error}")))?;
    if level_count == 0 || level_count > 253 {
        return Err(AppError::InvalidData(
            "river width count must be between 1 and 253".to_owned(),
        ));
    }
    let width_min = finite(&block, "WIDTH_MIN")?;
    let width_max = finite(&block, "WIDTH_MAX")?;
    if width_min <= 0.0 || width_max < width_min {
        return Err(AppError::InvalidData(
            "river width range is invalid".to_owned(),
        ));
    }
    Ok(RiverWidthMetadata {
        level_count,
        width_min,
        width_max,
    })
}

fn scalar<'a>(entries: &'a [Entry], key: &str) -> Result<&'a str, AppError> {
    let mut values = entries.iter().filter_map(|entry| match entry {
        Entry::Assignment(name, Value::Atom(value)) if name == key => Some(value.as_str()),
        _ => None,
    });
    let value = values
        .next()
        .ok_or_else(|| AppError::InvalidData(format!("river defines have no {key}")))?;
    if values.next().is_some() {
        return Err(AppError::InvalidData(format!("river defines repeat {key}")));
    }
    Ok(value)
}

fn finite(entries: &[Entry], key: &str) -> Result<f32, AppError> {
    let value = scalar(entries, key)?;
    let parsed = value
        .parse::<f32>()
        .map_err(|error| AppError::InvalidData(format!("invalid {key}: {error}")))?;
    if !parsed.is_finite() {
        return Err(AppError::InvalidData(format!("{key} must be finite")));
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::parse_river_widths;

    #[test]
    fn parses_required_width_values() {
        let input = b"NRivers={NUM_WIDTH_PIXEL_VALUES=13 WIDTH_MIN=1 WIDTH_MAX=2.2}";
        let parsed = parse_river_widths("test", input);
        assert_eq!(parsed.ok().map(|value| value.level_count), Some(13));
    }
}
