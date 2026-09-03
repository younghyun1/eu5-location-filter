//! Deterministic interning and final record assembly.

use std::collections::{BTreeSet, HashMap};

use super::hierarchy::RawHierarchy;
use super::templates::RawTemplate;
use crate::AppError;
use crate::model::{
    EU5_APP_ID, FORMAT_VERSION, Hierarchy, LocalizedValue, LocationId, LocationKind,
    LocationRecord, MapColor, RiverWidthMetadata, StoredDataset, StringInterner, SymbolId,
};

type RecordBuild = (
    StoredDataset,
    HashMap<String, LocationId>,
    HashMap<MapColor, LocationId>,
);

#[allow(clippy::too_many_arguments)]
pub(super) fn build_records(
    build_id: u64,
    river_widths: RiverWidthMetadata,
    templates: Vec<RawTemplate>,
    hierarchy: &mut HashMap<String, RawHierarchy>,
    colors: &mut HashMap<String, MapColor>,
    ports: &HashMap<String, String>,
    requested: &BTreeSet<String>,
    localization: &HashMap<String, String>,
) -> Result<RecordBuild, AppError> {
    let mut interner = StringInterner::new();
    let mut records = Vec::with_capacity(templates.len());
    let mut key_ids = HashMap::with_capacity(templates.len());
    let mut color_ids = HashMap::with_capacity(templates.len());
    for template in templates {
        let hierarchy_value = hierarchy.remove(&template.key).ok_or_else(|| {
            AppError::InvalidData(format!(
                "location {} lost hierarchy membership",
                template.key
            ))
        })?;
        let color = colors.remove(&template.key).ok_or_else(|| {
            AppError::InvalidData(format!("location {} lost its map color", template.key))
        })?;
        let id = LocationId(count_u32(records.len(), "locations")?);
        let key = interner.intern(&template.key)?;
        let fallback = title_case_identifier(&template.key);
        let name = interner.intern(
            localization
                .get(&template.key)
                .map_or(fallback.as_str(), String::as_str),
        )?;
        let record = LocationRecord {
            id,
            key,
            name,
            kind: LocationKind::from_topography(&template.topography),
            color,
            topography: interner.intern(&template.topography)?,
            vegetation: intern_option(&mut interner, template.vegetation.as_deref())?,
            climate: intern_option(&mut interner, template.climate.as_deref())?,
            religion: intern_option(&mut interner, template.religion.as_deref())?,
            culture: intern_option(&mut interner, template.culture.as_deref())?,
            raw_material: intern_option(&mut interner, template.raw_material.as_deref())?,
            modifier: intern_option(&mut interner, template.modifier.as_deref())?,
            harbor_suitability: template.harbor_suitability,
            movement_assistance: template.movement_assistance,
            hierarchy: intern_hierarchy(&mut interner, &hierarchy_value)?,
            coastal: ports.contains_key(&template.key),
            connected_sea: None,
            river: None,
            static_population_capacity: None,
        };
        key_ids.insert(template.key, id);
        color_ids.insert(color, id);
        records.push(record);
    }
    let mut localizations = Vec::new();
    for key in requested {
        // Each location record already carries its English-name symbol.
        if key_ids.contains_key(key) {
            continue;
        }
        if let Some(value) = localization.get(key) {
            localizations.push(LocalizedValue {
                key: interner.intern(key)?,
                value: interner.intern(value)?,
            });
        }
    }
    Ok((
        StoredDataset {
            format_version: FORMAT_VERSION,
            app_id: EU5_APP_ID,
            build_id,
            river_widths,
            dictionary: interner.into_values(),
            localizations,
            locations: records,
            diagnostics: Vec::new(),
        },
        key_ids,
        color_ids,
    ))
}

pub(super) fn resolve_ports(
    records: &mut [LocationRecord],
    key_ids: &HashMap<String, LocationId>,
    ports: &HashMap<String, String>,
) -> Result<(), AppError> {
    for (land, sea) in ports {
        let land_id = key_ids
            .get(land)
            .ok_or_else(|| AppError::InvalidData(format!("unknown port land location: {land}")))?;
        let sea_id = key_ids
            .get(sea)
            .ok_or_else(|| AppError::InvalidData(format!("unknown port sea location: {sea}")))?;
        let index = usize::try_from(land_id.0)
            .map_err(|error| AppError::InvalidData(format!("port index overflow: {error}")))?;
        let record = records
            .get_mut(index)
            .ok_or_else(|| AppError::InvalidData(format!("port index is invalid for {land}")))?;
        record.connected_sea = Some(*sea_id);
    }
    Ok(())
}

fn intern_option(
    interner: &mut StringInterner,
    value: Option<&str>,
) -> Result<Option<SymbolId>, AppError> {
    value.map(|value| interner.intern(value)).transpose()
}

fn intern_hierarchy(
    interner: &mut StringInterner,
    value: &RawHierarchy,
) -> Result<Hierarchy, AppError> {
    Ok(Hierarchy {
        continent: interner.intern(&value.continent)?,
        subcontinent: interner.intern(&value.subcontinent)?,
        region: interner.intern(&value.region)?,
        area: interner.intern(&value.area)?,
        province: interner.intern(&value.province)?,
    })
}

fn title_case_identifier(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut upper = true;
    for character in value.chars() {
        if matches!(character, '_' | '-') {
            output.push(' ');
            upper = true;
        } else if upper {
            output.extend(character.to_uppercase());
            upper = false;
        } else {
            output.push(character);
        }
    }
    output
}

pub(super) fn count_u32(value: usize, label: &str) -> Result<u32, AppError> {
    u32::try_from(value)
        .map_err(|error| AppError::InvalidData(format!("{label} count overflow: {error}")))
}

#[cfg(test)]
mod tests {
    use super::title_case_identifier;

    #[test]
    fn fallback_name_is_title_cased() {
        assert_eq!(
            title_case_identifier("north_eastern_atlantic"),
            "North Eastern Atlantic"
        );
        assert_eq!(title_case_identifier("éire"), "Éire");
    }
}
