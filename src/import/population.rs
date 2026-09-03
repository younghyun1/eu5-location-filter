//! Immutable population-capacity calculation from vanilla map factors.

use std::collections::HashMap;
use std::path::Path;

use crate::AppError;
use crate::model::{
    LocationKind, PopulationAmount, StaticPopulationCapacity, StoredDataset, SymbolId,
};
use crate::parser::{Entry, Value, parse_document, read_limited};
use crate::steam::GameInstallation;

const MAX_DEFINITION_SIZE: u64 = 4 * 1024 * 1024;
const PEOPLE_PER_CAPACITY_UNIT: f64 = 1_000.0;

pub(super) struct StaticFactors {
    vegetation: HashMap<String, f64>,
    topography: HashMap<String, f64>,
    climate: HashMap<String, f64>,
    equator_capacity: f64,
    coastal_modifier: f64,
    river_modifiers: [f64; 5],
    equator_y: f64,
}

pub(super) fn load(
    installation: &GameInstallation,
    equator_y: f64,
) -> Result<StaticFactors, AppError> {
    let vegetation = definitions(
        &installation.vegetation_definitions(),
        "local_population_capacity",
    )?;
    let topography = definitions(
        &installation.topography_definitions(),
        "local_population_capacity_modifier",
    )?;
    let climate = definitions(
        &installation.climate_definitions(),
        "local_population_capacity_modifier",
    )?;
    let static_path = installation.location_static_modifiers();
    let static_entries = document(&static_path)?;
    let equator_capacity = direct_effect(
        &static_entries,
        "location_closeness_to_equator_impact",
        "local_population_capacity",
    )?;
    let coastal_modifier = direct_effect(
        &static_entries,
        "coastal",
        "local_population_capacity_modifier",
    )?;
    let mut river_modifiers = [0.0; 5];
    for (index, value) in river_modifiers.iter_mut().enumerate() {
        *value = direct_effect(
            &static_entries,
            &format!("river_flowing_through_{}", index + 1),
            "local_population_capacity_modifier",
        )?;
    }
    Ok(StaticFactors {
        vegetation,
        topography,
        climate,
        equator_capacity,
        coastal_modifier,
        river_modifiers,
        equator_y,
    })
}

pub(super) fn calculate(
    stored: &mut StoredDataset,
    center_y: &[Option<f64>],
    factors: &StaticFactors,
) -> Result<(), AppError> {
    if center_y.len() != stored.locations.len() {
        return Err(AppError::InvalidData(
            "map-center count does not match location count".to_owned(),
        ));
    }
    let capacities = stored
        .locations
        .iter()
        .zip(center_y)
        .map(|(record, center)| {
            if record.kind != LocationKind::Land || record.vegetation.is_none() {
                return Ok(None);
            }
            let center = center.ok_or_else(|| {
                AppError::InvalidData(format!(
                    "location {} has no pixels in locations.png",
                    symbol(&stored.dictionary, Some(record.key)).unwrap_or("<invalid>")
                ))
            })?;
            let vegetation_key =
                symbol(&stored.dictionary, record.vegetation).ok_or_else(|| {
                    AppError::InvalidData("location has an invalid vegetation symbol".to_owned())
                })?;
            let topography_key =
                symbol(&stored.dictionary, Some(record.topography)).ok_or_else(|| {
                    AppError::InvalidData("location has an invalid topography symbol".to_owned())
                })?;
            let climate_key = symbol(&stored.dictionary, record.climate).ok_or_else(|| {
                AppError::InvalidData("land location has no valid climate symbol".to_owned())
            })?;
            let vegetation = required(&factors.vegetation, vegetation_key, "vegetation")?;
            let topography = required(&factors.topography, topography_key, "topography")?;
            let climate = required(&factors.climate, climate_key, "climate")?;
            let equator =
                factors.equator_capacity * closeness_to_equator(center, factors.equator_y);
            let river = record
                .river
                .and_then(|river| {
                    factors
                        .river_modifiers
                        .get(usize::from(river.level.0.saturating_sub(1)))
                })
                .copied()
                .unwrap_or(0.0);
            let modifier = topography
                + climate
                + if record.coastal {
                    factors.coastal_modifier
                } else {
                    0.0
                }
                + river;
            let base = (vegetation + equator) * PEOPLE_PER_CAPACITY_UNIT;
            let total = base * (1.0 + modifier);
            Ok(Some(StaticPopulationCapacity {
                vegetation: PopulationAmount(whole_people(
                    vegetation * PEOPLE_PER_CAPACITY_UNIT,
                    "vegetation capacity",
                )?),
                equator: PopulationAmount(whole_people(
                    equator * PEOPLE_PER_CAPACITY_UNIT,
                    "equator capacity",
                )?),
                modifier_basis_points: basis_points(modifier)?,
                total: PopulationAmount(whole_people(total, "static population capacity")?),
            }))
        })
        .collect::<Result<Vec<_>, AppError>>()?;
    for (record, capacity) in stored.locations.iter_mut().zip(capacities) {
        record.static_population_capacity = capacity;
    }
    Ok(())
}

fn definitions(path: &Path, effect: &str) -> Result<HashMap<String, f64>, AppError> {
    let entries = document(path)?;
    let mut values = HashMap::with_capacity(entries.len());
    for entry in entries {
        let Entry::Assignment(key, Value::Block(fields)) = entry else {
            continue;
        };
        let value = nested_effect(&fields, "location_modifier", effect)?.unwrap_or(0.0);
        if values.insert(key.clone(), value).is_some() {
            return Err(AppError::InvalidData(format!(
                "{} repeats definition {key}",
                path.display()
            )));
        }
    }
    Ok(values)
}

fn document(path: &Path) -> Result<Vec<Entry>, AppError> {
    parse_document(
        &path.display().to_string(),
        &read_limited(path, MAX_DEFINITION_SIZE)?,
    )
}

fn direct_effect(entries: &[Entry], block: &str, effect: &str) -> Result<f64, AppError> {
    let fields = entries.iter().find_map(|entry| match entry {
        Entry::Assignment(key, Value::Block(fields)) if key == block => Some(fields.as_slice()),
        _ => None,
    });
    let fields =
        fields.ok_or_else(|| AppError::InvalidData(format!("missing {block} modifier")))?;
    scalar_effect(fields, effect)?
        .ok_or_else(|| AppError::InvalidData(format!("modifier {block} has no {effect}")))
}

fn nested_effect(entries: &[Entry], block: &str, effect: &str) -> Result<Option<f64>, AppError> {
    let nested = entries.iter().find_map(|entry| match entry {
        Entry::Assignment(key, Value::Block(fields)) if key == block => Some(fields.as_slice()),
        _ => None,
    });
    nested.map_or(Ok(None), |fields| scalar_effect(fields, effect))
}

fn scalar_effect(entries: &[Entry], effect: &str) -> Result<Option<f64>, AppError> {
    let mut matches = entries.iter().filter_map(|entry| match entry {
        Entry::Assignment(key, Value::Atom(value)) if key == effect => Some(value.as_str()),
        _ => None,
    });
    let Some(value) = matches.next() else {
        return Ok(None);
    };
    if matches.next().is_some() {
        return Err(AppError::InvalidData(format!("modifier repeats {effect}")));
    }
    parse_finite(value).map(Some)
}

fn parse_finite(value: &str) -> Result<f64, AppError> {
    let parsed = value
        .parse::<f64>()
        .map_err(|error| AppError::InvalidData(format!("invalid modifier value: {error}")))?;
    if !parsed.is_finite() {
        return Err(AppError::InvalidData(
            "modifier value must be finite".to_owned(),
        ));
    }
    Ok(parsed)
}

fn closeness_to_equator(center_y: f64, equator_y: f64) -> f64 {
    (1.0 - (center_y - equator_y).abs() / equator_y).clamp(0.0, 1.0)
}

fn symbol(dictionary: &[String], id: Option<SymbolId>) -> Option<&str> {
    id.and_then(|id| usize::try_from(id.0).ok())
        .and_then(|index| dictionary.get(index))
        .map(String::as_str)
}

fn required(values: &HashMap<String, f64>, key: &str, kind: &str) -> Result<f64, AppError> {
    values
        .get(key)
        .copied()
        .ok_or_else(|| AppError::InvalidData(format!("{kind} definition is missing for {key}")))
}

fn whole_people(value: f64, label: &str) -> Result<u32, AppError> {
    if !value.is_finite() || value < 0.0 || value > f64::from(u32::MAX) {
        return Err(AppError::InvalidData(format!("{label} is out of range")));
    }
    Ok(value as u32)
}

fn basis_points(value: f64) -> Result<i16, AppError> {
    let value = (value * 10_000.0).round();
    if value < f64::from(i16::MIN) || value > f64::from(i16::MAX) {
        return Err(AppError::InvalidData(
            "static population modifier is out of range".to_owned(),
        ));
    }
    Ok(value as i16)
}

#[cfg(test)]
mod tests;
