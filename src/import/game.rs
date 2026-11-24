//! The location bundle import: map data, common definitions and English localization.

use std::collections::{HashMap, HashSet};

use super::hierarchy::{self, RawHierarchy};
use super::labels::{labelled_identifiers, requested_keys};
use super::progress::{ImportProgress, ImportStage, report};
use super::templates::{self, RawTemplate};
use super::{colors, map_definition, population, ports, records, river_defines, rivers};
use crate::AppError;
use crate::lang_pack::GameLanguage;
use crate::model::{ImportDiagnostic, StoredDataset};
use crate::parser::{read_limited, read_localizations};
use crate::steam::GameInstallation;

const MAX_TEMPLATES_SIZE: u64 = 16 * 1024 * 1024;
const MAX_DEFINITIONS_SIZE: u64 = 8 * 1024 * 1024;
const MAX_COLORS_SIZE: u64 = 8 * 1024 * 1024;
const MAX_PORTS_SIZE: u64 = 4 * 1024 * 1024;
const MAX_DEFINES_SIZE: u64 = 1024 * 1024;
/// Work units reported by the location import; `Complete` reports this value.
const STEPS: u64 = 10;

/// Imports all supported vanilla sources into a deterministic stored dataset.
///
/// Language packs are not built here; they pair with the written bundle, so the caller builds
/// them afterwards and reports `ImportStage::LanguagePacks` itself.
pub fn import_game(
    installation: &GameInstallation,
    mut progress: impl FnMut(ImportProgress),
) -> Result<StoredDataset, AppError> {
    let map = installation.map_data();
    report(&mut progress, ImportStage::LocationTemplates, 0, STEPS);
    let template_path = map.join("location_templates.txt");
    let templates = templates::parse_templates(
        &template_path.display().to_string(),
        &read_limited(&template_path, MAX_TEMPLATES_SIZE)?,
    )?;
    report(&mut progress, ImportStage::Hierarchy, 1, STEPS);
    let hierarchy_path = map.join("definitions.txt");
    let mut hierarchy = hierarchy::parse_hierarchy(
        &hierarchy_path.display().to_string(),
        &read_limited(&hierarchy_path, MAX_DEFINITIONS_SIZE)?,
    )?;
    report(&mut progress, ImportStage::MapColors, 2, STEPS);
    let colors_path = map.join("named_locations").join("00_default.txt");
    let mut colors = colors::parse_colors(
        &colors_path.display().to_string(),
        &read_limited(&colors_path, MAX_COLORS_SIZE)?,
    )?;
    report(&mut progress, ImportStage::Ports, 3, STEPS);
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
    report(&mut progress, ImportStage::RiverSettings, 4, STEPS);
    let river_defines_path = installation.river_defines();
    let river_widths = river_defines::parse_river_widths(
        &river_defines_path.display().to_string(),
        &read_limited(&river_defines_path, MAX_DEFINES_SIZE)?,
    )?;
    report(&mut progress, ImportStage::StaticMapFactors, 5, STEPS);
    let map_definition = map_definition::load(installation)?;
    let population_factors = population::load(installation, map_definition.equator_y)?;
    validate_source_coverage(&templates, &hierarchy, &colors, &ports)?;

    report(&mut progress, ImportStage::Localization, 6, STEPS);
    let labelled = labelled_identifiers(&templates, &hierarchy)?;
    let requested: HashSet<String> = requested_keys(&templates, &labelled).into_iter().collect();
    let localization = read_localizations(
        &installation.localization_roots(GameLanguage::English),
        &requested,
    )?;
    let (mut stored, key_ids, color_ids) = records::build_records(
        installation.build_id,
        river_widths,
        templates,
        &mut hierarchy,
        &mut colors,
        &ports,
        &map_definition.impassable,
        &labelled,
        &localization,
    )?;

    report(&mut progress, ImportStage::MapAndRivers, 7, STEPS);
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
    report(&mut progress, ImportStage::PopulationCapacity, 8, STEPS);
    population::calculate(&mut stored, &river_scan.center_y, &population_factors)?;
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
    report(&mut progress, ImportStage::Complete, STEPS, STEPS);
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
