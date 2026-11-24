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
                            let location = match entry {
                                Entry::Atom(location) => location,
                                // EU5 1.4 writes scalar attributes such as
                                // `not_eligible_for_dynamic_country_name = yes` beside
                                // the member list; they carry no membership.
                                Entry::Assignment(_, Value::Atom(_)) => continue,
                                Entry::Assignment(_, Value::Block(_)) => {
                                    return Err(AppError::InvalidData(format!(
                                        "province {province} contains a nested block"
                                    )));
                                }
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

/// Returns the child blocks of one hierarchy level.
///
/// Scalar assignments are attributes of the enclosing level (EU5 1.4 adds
/// `not_eligible_for_dynamic_country_name = yes` to areas and provinces) and
/// are skipped; a bare value at a level that only holds named blocks is malformed.
fn assignments(entries: Vec<Entry>, level: &str) -> Result<Vec<(String, Vec<Entry>)>, AppError> {
    let mut output = Vec::with_capacity(entries.len());
    for entry in entries {
        match entry {
            Entry::Assignment(key, Value::Block(values)) => output.push((key, values)),
            Entry::Assignment(_, Value::Atom(_)) => {}
            Entry::Atom(value) => {
                return Err(AppError::InvalidData(format!(
                    "unexpected bare value {value} at {level} level"
                )));
            }
        }
    }
    Ok(output)
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
    fn skips_scalar_attributes_at_area_and_province_levels() {
        let input = b"c={s={r={a={flag=yes p={flag=yes one two} q={three}}}}}";
        let parsed = parse_hierarchy("test", input);
        assert!(parsed.is_ok());
        let Ok(parsed) = parsed else { return };
        assert_eq!(parsed.len(), 3);
        assert_eq!(
            parsed.get("one").map(|value| value.province.as_str()),
            Some("p")
        );
        assert!(!parsed.contains_key("flag"));
    }

    #[test]
    fn rejects_duplicate_membership() {
        let input = b"c={s={r={a={p={one} q={one}}}}}";
        assert!(parse_hierarchy("test", input).is_err());
    }
}
