//! Location-template extraction.

use std::collections::HashSet;

use crate::AppError;
use crate::model::MAX_LOCATIONS;
use crate::parser::{Entry, Value, parse_document};

/// Uninterned location template.
#[derive(Clone, Debug)]
pub(super) struct RawTemplate {
    pub key: String,
    pub topography: String,
    pub vegetation: Option<String>,
    pub climate: Option<String>,
    pub religion: Option<String>,
    pub culture: Option<String>,
    pub raw_material: Option<String>,
    pub modifier: Option<String>,
    pub harbor_suitability: Option<f32>,
    pub movement_assistance: Option<[f32; 2]>,
}

pub(super) fn parse_templates(
    source_name: &str,
    input: &[u8],
) -> Result<Vec<RawTemplate>, AppError> {
    let entries = parse_document(source_name, input)?;
    if entries.len() > MAX_LOCATIONS {
        return Err(AppError::InvalidData(format!(
            "location count {} exceeds limit {MAX_LOCATIONS}",
            entries.len()
        )));
    }
    let mut keys = HashSet::with_capacity(entries.len());
    let mut output = Vec::with_capacity(entries.len());
    for entry in entries {
        let Entry::Assignment(key, Value::Block(fields)) = entry else {
            return Err(AppError::InvalidData(
                "every location template must be an assignment block".to_owned(),
            ));
        };
        if !keys.insert(key.clone()) {
            return Err(AppError::InvalidData(format!(
                "duplicate location identifier: {key}"
            )));
        }
        output.push(parse_template(key, fields)?);
    }
    Ok(output)
}

fn parse_template(key: String, fields: Vec<Entry>) -> Result<RawTemplate, AppError> {
    let mut seen = HashSet::new();
    let mut template = RawTemplate {
        key: key.clone(),
        topography: String::new(),
        vegetation: None,
        climate: None,
        religion: None,
        culture: None,
        raw_material: None,
        modifier: None,
        harbor_suitability: None,
        movement_assistance: None,
    };
    for field in fields {
        let Entry::Assignment(name, value) = field else {
            return Err(AppError::InvalidData(format!(
                "bare value in location template {key}"
            )));
        };
        if !seen.insert(name.clone()) {
            return Err(AppError::InvalidData(format!(
                "duplicate field {name} in location {key}"
            )));
        }
        match name.as_str() {
            "topography" => template.topography = atom(&key, &name, value)?,
            "vegetation" => template.vegetation = Some(atom(&key, &name, value)?),
            "climate" => template.climate = Some(atom(&key, &name, value)?),
            "religion" => template.religion = Some(atom(&key, &name, value)?),
            "culture" => template.culture = Some(atom(&key, &name, value)?),
            "raw_material" => template.raw_material = Some(atom(&key, &name, value)?),
            "modifier" => template.modifier = Some(atom(&key, &name, value)?),
            "natural_harbor_suitability" => {
                let raw = atom(&key, &name, value)?;
                template.harbor_suitability = Some(parse_finite(&key, &name, &raw)?);
            }
            "movement_assistance" => {
                template.movement_assistance = Some(parse_vector(&key, value)?)
            }
            _ => {}
        }
    }
    if template.topography.is_empty() {
        return Err(AppError::InvalidData(format!(
            "location {key} has no topography"
        )));
    }
    Ok(template)
}

fn atom(location: &str, field: &str, value: Value) -> Result<String, AppError> {
    match value {
        Value::Atom(value) if !value.is_empty() => Ok(value),
        Value::Atom(_) | Value::Block(_) => Err(AppError::InvalidData(format!(
            "location {location} field {field} must be a non-empty scalar"
        ))),
    }
}

fn parse_finite(location: &str, field: &str, value: &str) -> Result<f32, AppError> {
    let parsed = value.parse::<f32>().map_err(|error| {
        AppError::InvalidData(format!(
            "location {location} field {field} is not numeric: {error}"
        ))
    })?;
    if !parsed.is_finite() {
        return Err(AppError::InvalidData(format!(
            "location {location} field {field} must be finite"
        )));
    }
    Ok(parsed)
}

fn parse_vector(location: &str, value: Value) -> Result<[f32; 2], AppError> {
    let Value::Block(entries) = value else {
        return Err(AppError::InvalidData(format!(
            "location {location} movement assistance must be a block"
        )));
    };
    let values = entries
        .into_iter()
        .map(|entry| match entry {
            Entry::Atom(value) => parse_finite(location, "movement_assistance", &value),
            Entry::Assignment(_, _) => Err(AppError::InvalidData(format!(
                "location {location} movement assistance must contain two numbers"
            ))),
        })
        .collect::<Result<Vec<_>, _>>()?;
    match values.as_slice() {
        [first, second] => Ok([*first, *second]),
        _ => Err(AppError::InvalidData(format!(
            "location {location} movement assistance must contain exactly two numbers"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_templates;

    #[test]
    fn parses_optional_fields_and_vector() {
        let input = b"ocean={topography=deep_ocean movement_assistance={-1.5 2.25}}";
        let parsed = parse_templates("test", input);
        assert!(parsed.is_ok());
        let Ok(values) = parsed else { return };
        assert_eq!(
            values.first().and_then(|value| value.movement_assistance),
            Some([-1.5, 2.25])
        );
    }

    #[test]
    fn rejects_duplicate_ids_and_bad_numbers() {
        assert!(parse_templates("test", b"a={topography=flatland} a={topography=hills}").is_err());
        assert!(
            parse_templates(
                "test",
                b"a={topography=flatland natural_harbor_suitability=no}"
            )
            .is_err()
        );
    }
}
