//! Static map metadata needed during location classification.

use std::collections::HashSet;

use crate::AppError;
use crate::parser::{Entry, Value, parse_document, read_limited};
use crate::steam::GameInstallation;

const MAX_MAP_DEFINITION_SIZE: u64 = 4 * 1024 * 1024;

pub(super) struct MapDefinition {
    pub equator_y: f64,
    pub impassable: HashSet<String>,
}

pub(super) fn load(installation: &GameInstallation) -> Result<MapDefinition, AppError> {
    let path = installation.map_definition();
    let entries = parse_document(
        &path.display().to_string(),
        &read_limited(&path, MAX_MAP_DEFINITION_SIZE)?,
    )?;
    parse(&entries)
}

fn parse(entries: &[Entry]) -> Result<MapDefinition, AppError> {
    let equator = entries.iter().find_map(|entry| match entry {
        Entry::Assignment(key, Value::Atom(value)) if key == "equator_y" => Some(value.as_str()),
        _ => None,
    });
    let equator_y = equator
        .ok_or_else(|| AppError::InvalidData("default.map has no equator_y".to_owned()))?
        .parse::<f64>()
        .map_err(|error| AppError::InvalidData(format!("invalid equator_y: {error}")))?;
    if !equator_y.is_finite() || equator_y <= 0.0 {
        return Err(AppError::InvalidData(
            "equator_y must be finite and positive".to_owned(),
        ));
    }
    let values = entries.iter().find_map(|entry| match entry {
        Entry::Assignment(key, Value::Block(values)) if key == "impassable_mountains" => {
            Some(values.as_slice())
        }
        _ => None,
    });
    let values = values.ok_or_else(|| {
        AppError::InvalidData("default.map has no impassable_mountains block".to_owned())
    })?;
    let mut impassable = HashSet::with_capacity(values.len());
    for value in values {
        let Entry::Atom(key) = value else {
            return Err(AppError::InvalidData(
                "impassable_mountains must contain location identifiers".to_owned(),
            ));
        };
        impassable.insert(key.clone());
    }
    Ok(MapDefinition {
        equator_y,
        impassable,
    })
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::parser::parse_document;

    #[test]
    fn parses_equator_and_explicit_impassables() {
        let parsed = parse_document(
            "test",
            b"equator_y=3340 impassable_mountains={heard_island flatland_wasteland}",
        );
        assert!(parsed.is_ok());
        let Ok(entries) = parsed else { return };
        let definition = parse(&entries);
        assert!(definition.is_ok());
        let Ok(definition) = definition else { return };
        assert_eq!(definition.equator_y, 3_340.0);
        assert!(definition.impassable.contains("heard_island"));
    }
}
