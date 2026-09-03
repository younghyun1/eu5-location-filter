//! Vanilla EU5 installation importer.

mod colors;
mod hierarchy;
mod ports;
mod river_defines;
mod rivers;
mod templates;

use std::collections::{BTreeSet, HashMap, HashSet};

use crate::AppError;
use crate::model::{
    EU5_APP_ID, FORMAT_VERSION, Hierarchy, ImportDiagnostic, LocalizedValue, LocationId,
    LocationKind, LocationRecord, StoredDataset, StringInterner, SymbolId,
};
use crate::parser::{read_limited, read_localizations};
use crate::steam::GameInstallation;
use hierarchy::RawHierarchy;
use templates::RawTemplate;

const MAX_TEMPLATES_SIZE: u64 = 16 * 1024 * 1024;
const MAX_DEFINITIONS_SIZE: u64 = 8 * 1024 * 1024;
const MAX_COLORS_SIZE: u64 = 8 * 1024 * 1024;
const MAX_PORTS_SIZE: u64 = 4 * 1024 * 1024;
const MAX_DEFINES_SIZE: u64 = 1024 * 1024;

/// Import progress suitable for either a terminal or the Slint event loop.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ImportProgress {
    /// Stable stage label.
    pub stage: &'static str,
    /// Completed work units.
    pub current: u64,
    /// Total work units when known.
    pub total: u64,
}

/// Imports all supported vanilla sources into a deterministic stored dataset.
pub fn import_game(
    installation: &GameInstallation,
    mut progress: impl FnMut(ImportProgress),
) -> Result<StoredDataset, AppError> {
    let map = installation.map_data();
    progress_at(&mut progress, "Reading location templates", 0, 8);
    let template_path = map.join("location_templates.txt");
    let templates = templates::parse_templates(
        &template_path.display().to_string(),
        &read_limited(&template_path, MAX_TEMPLATES_SIZE)?,
    )?;
    progress_at(&mut progress, "Reading hierarchy", 1, 8);
    let hierarchy_path = map.join("definitions.txt");
    let mut hierarchy = hierarchy::parse_hierarchy(
        &hierarchy_path.display().to_string(),
        &read_limited(&hierarchy_path, MAX_DEFINITIONS_SIZE)?,
    )?;
    progress_at(&mut progress, "Reading map colors", 2, 8);
    let colors_path = map.join("named_locations").join("00_default.txt");
    let mut colors = colors::parse_colors(
        &colors_path.display().to_string(),
        &read_limited(&colors_path, MAX_COLORS_SIZE)?,
    )?;
    progress_at(&mut progress, "Reading ports", 3, 8);
    let ports_path = map.join("ports.csv");
    let ports_bytes = read_limited(&ports_path, MAX_PORTS_SIZE)?;
    let ports_text = String::from_utf8(ports_bytes).map_err(|error| {
        AppError::parse(
            ports_path.display().to_string(),
            0,
            format!("invalid UTF-8: {error}"),
        )
    })?;
    let ports = ports::parse_ports(&ports_text)?;
    progress_at(&mut progress, "Reading river settings", 4, 8);
    let river_defines_path = installation.river_defines();
    let river_widths = river_defines::parse_river_widths(
        &river_defines_path.display().to_string(),
        &read_limited(&river_defines_path, MAX_DEFINES_SIZE)?,
    )?;
    validate_source_coverage(&templates, &hierarchy, &colors, &ports)?;

    progress_at(&mut progress, "Reading English localization", 5, 8);
    let requested = referenced_symbols(&templates, &hierarchy);
    let localization = read_localizations(
        &installation.localization_roots(),
        &requested.iter().cloned().collect(),
    )?;
    let (mut stored, key_ids, color_ids) = build_records(
        installation.build_id,
        river_widths,
        templates,
        &mut hierarchy,
        &mut colors,
        &ports,
        &requested,
        &localization,
    )?;

    progress_at(&mut progress, "Scanning map and rivers", 6, 8);
    let river_scan = rivers::scan_rivers(
        &map.join("locations.png"),
        &map.join("rivers.png"),
        &color_ids,
        stored.locations.len(),
        river_widths,
    )?;
    for (record, river) in stored.locations.iter_mut().zip(river_scan.values) {
        record.river = river;
    }
    resolve_ports(&mut stored.locations, &key_ids, &ports)?;
    if !colors.is_empty() {
        stored.diagnostics.push(ImportDiagnostic {
            code: "unused_named_colors".to_owned(),
            count: count_u32(colors.len(), "unused named colors")?,
        });
    }
    if river_scan.unknown_pixels > 0 {
        stored.diagnostics.push(ImportDiagnostic {
            code: "unknown_river_location_pixels".to_owned(),
            count: u32::try_from(river_scan.unknown_pixels).unwrap_or(u32::MAX),
        });
    }
    progress_at(&mut progress, "Import complete", 8, 8);
    Ok(stored)
}

fn validate_source_coverage(
    templates: &[RawTemplate],
    hierarchy: &HashMap<String, RawHierarchy>,
    colors: &HashMap<String, crate::model::MapColor>,
    ports: &HashMap<String, String>,
) -> Result<(), AppError> {
    let keys: HashSet<&str> = templates
        .iter()
        .map(|template| template.key.as_str())
        .collect();
    for template in templates {
        if !hierarchy.contains_key(&template.key) {
            return Err(AppError::InvalidData(format!(
                "location {} has no hierarchy membership",
                template.key
            )));
        }
        if !colors.contains_key(&template.key) {
            return Err(AppError::InvalidData(format!(
                "location {} has no named map color",
                template.key
            )));
        }
    }
    for (land, sea) in ports {
        if !keys.contains(land.as_str()) || !keys.contains(sea.as_str()) {
            return Err(AppError::InvalidData(format!(
                "port {land} references unknown sea location {sea}"
            )));
        }
    }
    Ok(())
}

fn referenced_symbols(
    templates: &[RawTemplate],
    hierarchy: &HashMap<String, RawHierarchy>,
) -> BTreeSet<String> {
    let mut values = BTreeSet::new();
    for template in templates {
        values.insert(template.key.clone());
        values.insert(template.topography.clone());
        for value in [
            &template.vegetation,
            &template.climate,
            &template.religion,
            &template.culture,
            &template.raw_material,
            &template.modifier,
        ]
        .into_iter()
        .flatten()
        {
            values.insert(value.clone());
        }
        if let Some(value) = hierarchy.get(&template.key) {
            values.extend([
                value.continent.clone(),
                value.subcontinent.clone(),
                value.region.clone(),
                value.area.clone(),
                value.province.clone(),
            ]);
        }
    }
    values
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn build_records(
    build_id: u64,
    river_widths: crate::model::RiverWidthMetadata,
    templates: Vec<RawTemplate>,
    hierarchy: &mut HashMap<String, RawHierarchy>,
    colors: &mut HashMap<String, crate::model::MapColor>,
    ports: &HashMap<String, String>,
    requested: &BTreeSet<String>,
    localization: &HashMap<String, String>,
) -> Result<
    (
        StoredDataset,
        HashMap<String, LocationId>,
        HashMap<crate::model::MapColor, LocationId>,
    ),
    AppError,
> {
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
        };
        key_ids.insert(template.key, id);
        color_ids.insert(color, id);
        records.push(record);
    }
    let mut localizations = Vec::new();
    for key in requested {
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

fn resolve_ports(
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

fn count_u32(value: usize, label: &str) -> Result<u32, AppError> {
    u32::try_from(value)
        .map_err(|error| AppError::InvalidData(format!("{label} count overflow: {error}")))
}

fn progress_at(
    progress: &mut impl FnMut(ImportProgress),
    stage: &'static str,
    current: u64,
    total: u64,
) {
    progress(ImportProgress {
        stage,
        current,
        total,
    });
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
