//! Vanilla EU5 installation importer.

mod colors;
mod hierarchy;
mod ports;
mod records;
mod river_defines;
mod rivers;
mod templates;

use std::collections::{BTreeSet, HashMap, HashSet};

use crate::AppError;
use crate::model::{ImportDiagnostic, StoredDataset};
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
    let (mut stored, key_ids, color_ids) = records::build_records(
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
    records::resolve_ports(&mut stored.locations, &key_ids, &ports)?;
    if !colors.is_empty() {
        stored.diagnostics.push(ImportDiagnostic {
            code: "unused_named_colors".to_owned(),
            count: records::count_u32(colors.len(), "unused named colors")?,
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
