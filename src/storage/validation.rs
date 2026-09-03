//! Stored dataset invariant checks.

use std::collections::HashSet;

use crate::AppError;
use crate::model::{
    EU5_APP_ID, FORMAT_VERSION, LocationRecord, MAX_DICTIONARY_BYTES, MAX_LOCATIONS, MAX_SYMBOLS,
    StoredDataset, SymbolId,
};

pub(super) fn validate_stored(stored: &StoredDataset) -> Result<(), AppError> {
    if stored.format_version != FORMAT_VERSION {
        return Err(AppError::InvalidData(format!(
            "unsupported schema version {}",
            stored.format_version
        )));
    }
    if stored.app_id != EU5_APP_ID {
        return Err(AppError::InvalidData(format!(
            "unexpected Steam app ID {}",
            stored.app_id
        )));
    }
    if stored.dictionary.len() > MAX_SYMBOLS
        || stored.dictionary.iter().map(String::len).sum::<usize>() > MAX_DICTIONARY_BYTES
    {
        return Err(AppError::InvalidData(
            "dictionary exceeds configured limits".to_owned(),
        ));
    }
    if stored.locations.len() > MAX_LOCATIONS || stored.locations.is_empty() {
        return Err(AppError::InvalidData(
            "location count is outside configured limits".to_owned(),
        ));
    }
    let symbol_count = stored.dictionary.len();
    let location_count = stored.locations.len();
    let mut keys = HashSet::with_capacity(location_count);
    let mut colors = HashSet::with_capacity(location_count);
    for (index, record) in stored.locations.iter().enumerate() {
        validate_record(
            record,
            index,
            symbol_count,
            location_count,
            stored.river_widths.level_count,
        )?;
        if !keys.insert(record.key) || !colors.insert(record.color) {
            return Err(AppError::InvalidData(
                "location identifiers and colors must be unique".to_owned(),
            ));
        }
    }
    for value in &stored.localizations {
        validate_symbol(value.key, symbol_count)?;
        validate_symbol(value.value, symbol_count)?;
    }
    if stored.river_widths.level_count == 0
        || !stored.river_widths.width_min.is_finite()
        || !stored.river_widths.width_max.is_finite()
        || stored.river_widths.width_min <= 0.0
        || stored.river_widths.width_max < stored.river_widths.width_min
    {
        return Err(AppError::InvalidData(
            "river width metadata is invalid".to_owned(),
        ));
    }
    Ok(())
}

fn validate_record(
    record: &LocationRecord,
    index: usize,
    symbol_count: usize,
    location_count: usize,
    max_river_level: u8,
) -> Result<(), AppError> {
    if usize::try_from(record.id.0).ok() != Some(index) {
        return Err(AppError::InvalidData(
            "location IDs must match storage order".to_owned(),
        ));
    }
    for symbol in symbols(record) {
        validate_symbol(symbol, symbol_count)?;
    }
    if record
        .harbor_suitability
        .is_some_and(|value| !value.is_finite())
        || record
            .movement_assistance
            .is_some_and(|value| value.into_iter().any(|component| !component.is_finite()))
    {
        return Err(AppError::InvalidData(
            "location contains a non-finite number".to_owned(),
        ));
    }
    if record
        .connected_sea
        .is_some_and(|value| usize::try_from(value.0).map_or(true, |value| value >= location_count))
    {
        return Err(AppError::InvalidData(
            "connected sea location is out of range".to_owned(),
        ));
    }
    if record.river.is_some_and(|river| {
        river.level.0 > max_river_level
            || !river.rendered_width.is_finite()
            || river.rendered_width < 0.0
    }) {
        return Err(AppError::InvalidData("river data is invalid".to_owned()));
    }
    Ok(())
}

fn symbols(record: &LocationRecord) -> impl Iterator<Item = SymbolId> {
    [
        Some(record.key),
        Some(record.name),
        Some(record.topography),
        record.vegetation,
        record.climate,
        record.religion,
        record.culture,
        record.raw_material,
        record.modifier,
        Some(record.hierarchy.continent),
        Some(record.hierarchy.subcontinent),
        Some(record.hierarchy.region),
        Some(record.hierarchy.area),
        Some(record.hierarchy.province),
    ]
    .into_iter()
    .flatten()
}

fn validate_symbol(symbol: SymbolId, count: usize) -> Result<(), AppError> {
    if usize::try_from(symbol.0).map_or(true, |value| value >= count) {
        return Err(AppError::InvalidData(
            "dictionary symbol is out of range".to_owned(),
        ));
    }
    Ok(())
}
