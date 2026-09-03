//! Five-level geography hierarchy extraction.

use std::collections::HashMap;

use crate::AppError;
use crate::parser::{Entry, Value, parse_document};

/// Uninterned hierarchy membership.
#[derive(Clone, Debug)]
pub(super) struct RawHierarchy {
    pub continent: String,
    pub subcontinent: String,
    pub region: String,
    pub area: String,
    pub province: String,
}

pub(super) fn parse_hierarchy(
    source_name: &str,
    input: &[u8],
) -> Result<HashMap<String, RawHierarchy>, AppError> {
    let entries = parse_document(source_name, input)?;
    let mut output = HashMap::new();
    for (continent, subcontinents) in assignments(entries, "continent")? {
        for (subcontinent, regions) in assignments(subcontinents, "subcontinent")? {
            for (region, areas) in assignments(regions, "region")? {
                for (area, provinces) in assignments(areas, "area")? {
                    for (province, locations) in assignments(provinces, "province")? {
                        for entry in locations {
                            let Entry::Atom(location) = entry else {
                                return Err(AppError::InvalidData(format!(
                                    "province {province} contains a nested assignment"
                                )));
                            };
                            let hierarchy = RawHierarchy {
                                continent: continent.clone(),
                                subcontinent: subcontinent.clone(),
                                region: region.clone(),
                                area: area.clone(),
                                province: province.clone(),
                            };
                            if output.insert(location.clone(), hierarchy).is_some() {
                                return Err(AppError::InvalidData(format!(
                                    "location {location} has duplicate hierarchy membership"
                                )));
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(output)
}

fn assignments(entries: Vec<Entry>, level: &str) -> Result<Vec<(String, Vec<Entry>)>, AppError> {
    entries
        .into_iter()
        .map(|entry| match entry {
            Entry::Assignment(key, Value::Block(values)) => Ok((key, values)),
            Entry::Assignment(key, Value::Atom(_)) => Err(AppError::InvalidData(format!(
                "{level} {key} must contain a block"
            ))),
            Entry::Atom(value) => Err(AppError::InvalidData(format!(
                "unexpected bare value {value} at {level} level"
            ))),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse_hierarchy;

    #[test]
    fn parses_multiline_membership() {
        let input = b"c={s={r={a={p={one two}}}}}";
        let parsed = parse_hierarchy("test", input);
        assert!(parsed.is_ok());
        assert_eq!(parsed.ok().map(|value| value.len()), Some(2));
    }

    #[test]
    fn rejects_duplicate_membership() {
        let input = b"c={s={r={a={p={one} q={one}}}}}";
        assert!(parse_hierarchy("test", input).is_err());
    }
}
