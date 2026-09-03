//! Coastal-location extraction from `ports.csv`.

use std::collections::HashMap;

use crate::AppError;
use crate::parser::parse_semicolon_line;

pub(super) fn parse_ports(input: &str) -> Result<HashMap<String, String>, AppError> {
    let mut lines = input.lines();
    let Some(header) = lines.next() else {
        return Err(AppError::InvalidData("ports.csv is empty".to_owned()));
    };
    let columns = parse_semicolon_line(header)?;
    if columns.first().map(String::as_str) != Some("LandProvince")
        || columns.get(1).map(String::as_str) != Some("SeaZone")
    {
        return Err(AppError::InvalidData(
            "ports.csv has an unsupported header".to_owned(),
        ));
    }
    let mut output = HashMap::new();
    for (index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let fields = parse_semicolon_line(line)?;
        let land = fields
            .first()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                AppError::InvalidData(format!("ports.csv row {} has no land location", index + 2))
            })?;
        let sea = fields
            .get(1)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                AppError::InvalidData(format!("ports.csv row {} has no sea location", index + 2))
            })?;
        if output.insert(land.clone(), sea.clone()).is_some() {
            return Err(AppError::InvalidData(format!(
                "duplicate port entry for {land}"
            )));
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::parse_ports;

    #[test]
    fn parses_optional_trailing_columns() {
        let input = "LandProvince;SeaZone;x;y;\nstockholm;sea;1;2;x\n";
        assert_eq!(
            parse_ports(input)
                .ok()
                .and_then(|map| map.get("stockholm").cloned()),
            Some("sea".to_owned())
        );
    }
}
