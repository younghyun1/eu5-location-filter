//! Row-streamed location and river image pairing.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Seek};
use std::path::Path;

use png::{BitDepth, ColorType, Decoder};

use crate::AppError;
use crate::model::{
    LocationId, MAX_RIVER_LEVEL, MapColor, RiverData, RiverLevel, RiverWidthMetadata,
};

const MAP_WIDTH: u32 = 16_384;
const MAP_HEIGHT: u32 = 8_192;
const MAX_PNG_SIZE: u64 = 256 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Default)]
struct Accumulator {
    present: bool,
    source: bool,
    confluence: bool,
    level: u8,
}

#[derive(Clone, Copy, Debug)]
struct PositionAccumulator {
    present: bool,
    minimum_y: u32,
    maximum_y: u32,
}

impl Default for PositionAccumulator {
    fn default() -> Self {
        Self {
            present: false,
            minimum_y: u32::MAX,
            maximum_y: 0,
        }
    }
}

/// Result of the paired image pass.
pub(super) struct RiverScan {
    pub values: Vec<Option<RiverData>>,
    pub center_y: Vec<Option<f64>>,
    pub unknown_pixels: u64,
}

/// Streams both production images without retaining an uncompressed frame.
pub(super) fn scan_rivers(
    locations_path: &Path,
    rivers_path: &Path,
    by_color: &HashMap<MapColor, LocationId>,
    location_count: usize,
    widths: RiverWidthMetadata,
) -> Result<RiverScan, AppError> {
    check_size(locations_path)?;
    check_size(rivers_path)?;
    let locations = File::open(locations_path)
        .map(BufReader::new)
        .map_err(|source| AppError::io("open", locations_path, source))?;
    let rivers = File::open(rivers_path)
        .map(BufReader::new)
        .map_err(|source| AppError::io("open", rivers_path, source))?;
    scan_readers(
        locations,
        rivers,
        locations_path,
        rivers_path,
        by_color,
        location_count,
        widths,
        Some((MAP_WIDTH, MAP_HEIGHT)),
    )
}

fn check_size(path: &Path) -> Result<(), AppError> {
    let size = fs::metadata(path)
        .map_err(|source| AppError::io("inspect", path, source))?
        .len();
    if size > MAX_PNG_SIZE {
        return Err(AppError::InvalidData(format!(
            "PNG exceeds compressed size limit: {}",
            path.display()
        )));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn scan_readers<L: BufRead + Seek, R: BufRead + Seek>(
    locations_input: L,
    rivers_input: R,
    locations_path: &Path,
    rivers_path: &Path,
    by_color: &HashMap<MapColor, LocationId>,
    location_count: usize,
    widths: RiverWidthMetadata,
    expected_dimensions: Option<(u32, u32)>,
) -> Result<RiverScan, AppError> {
    let mut locations = Decoder::new(locations_input)
        .read_info()
        .map_err(|source| AppError::Png {
            path: locations_path.to_owned(),
            source,
        })?;
    let mut rivers = Decoder::new(rivers_input)
        .read_info()
        .map_err(|source| AppError::Png {
            path: rivers_path.to_owned(),
            source,
        })?;
    let location_info = locations.info();
    let river_info = rivers.info();
    let dimensions = (location_info.width, location_info.height);
    if dimensions != (river_info.width, river_info.height) {
        return Err(AppError::InvalidData(
            "locations.png and rivers.png dimensions differ".to_owned(),
        ));
    }
    if expected_dimensions.is_some_and(|expected| expected != dimensions) {
        return Err(AppError::InvalidData(format!(
            "map dimensions are {}x{}; expected {}x{}",
            dimensions.0,
            dimensions.1,
            expected_dimensions.map_or(0, |value| value.0),
            expected_dimensions.map_or(0, |value| value.1)
        )));
    }
    if location_info.interlaced || river_info.interlaced {
        return Err(AppError::InvalidData(
            "interlaced map images are unsupported".to_owned(),
        ));
    }
    if locations.output_color_type() != (ColorType::Rgb, BitDepth::Eight) {
        return Err(AppError::InvalidData(
            "locations.png must be 8-bit RGB".to_owned(),
        ));
    }
    if rivers.output_color_type() != (ColorType::Indexed, BitDepth::Eight) {
        return Err(AppError::InvalidData(
            "rivers.png must preserve 8-bit palette indexes".to_owned(),
        ));
    }
    let width = usize::try_from(dimensions.0)
        .map_err(|error| AppError::InvalidData(format!("map width overflow: {error}")))?;
    let height = usize::try_from(dimensions.1)
        .map_err(|error| AppError::InvalidData(format!("map height overflow: {error}")))?;
    let mut accumulators = vec![Accumulator::default(); location_count];
    let mut positions = vec![PositionAccumulator::default(); location_count];
    let mut unknown_pixels = 0_u64;
    for row_index in 0..height {
        let location_row = next_row(&mut locations, locations_path)?;
        let river_row = next_row(&mut rivers, rivers_path)?;
        if location_row.len() != width.saturating_mul(3) || river_row.len() != width {
            return Err(AppError::InvalidData(
                "decoded PNG row has an unexpected length".to_owned(),
            ));
        }
        let (location_pixels, remainder) = location_row.as_chunks::<3>();
        if !remainder.is_empty() {
            return Err(AppError::InvalidData(
                "locations.png row has partial RGB data".to_owned(),
            ));
        }
        let map_y = u32::try_from(height.saturating_sub(row_index).saturating_sub(1))
            .map_err(|error| AppError::InvalidData(format!("map row overflow: {error}")))?;
        for (rgb, palette) in location_pixels.iter().zip(river_row.iter().copied()) {
            let color =
                MapColor((u32::from(rgb[0]) << 16) | (u32::from(rgb[1]) << 8) | u32::from(rgb[2]));
            let Some(location_id) = by_color.get(&color) else {
                if palette > 0 && palette <= widths.level_count.saturating_add(2) {
                    unknown_pixels = unknown_pixels.saturating_add(1);
                }
                continue;
            };
            let Ok(index) = usize::try_from(location_id.0) else {
                return Err(AppError::InvalidData(
                    "location index does not fit this platform".to_owned(),
                ));
            };
            let Some(position) = positions.get_mut(index) else {
                return Err(AppError::InvalidData(
                    "location image resolved an invalid location index".to_owned(),
                ));
            };
            position.present = true;
            position.minimum_y = position.minimum_y.min(map_y);
            position.maximum_y = position.maximum_y.max(map_y);
            if palette == 0 || palette > widths.level_count.saturating_add(2) {
                continue;
            }
            let Some(accumulator) = accumulators.get_mut(index) else {
                return Err(AppError::InvalidData(
                    "river image resolved an invalid location index".to_owned(),
                ));
            };
            accumulator.present = true;
            match palette {
                1 => accumulator.source = true,
                2 => accumulator.confluence = true,
                value => accumulator.level = accumulator.level.max(value - 2),
            }
        }
    }
    if next_optional_row(&mut locations, locations_path)?.is_some()
        || next_optional_row(&mut rivers, rivers_path)?.is_some()
    {
        return Err(AppError::InvalidData(
            "decoded PNG has more rows than its header".to_owned(),
        ));
    }
    let values = accumulators
        .into_iter()
        .map(|value| {
            value.present.then(|| RiverData {
                level: gameplay_level(value.level),
                has_source: value.source,
                has_confluence: value.confluence,
            })
        })
        .collect();
    let center_y = positions
        .into_iter()
        .map(|position| {
            position
                .present
                .then(|| (f64::from(position.minimum_y) + f64::from(position.maximum_y)) / 2.0)
        })
        .collect();
    Ok(RiverScan {
        values,
        center_y,
        unknown_pixels,
    })
}

fn next_row<'a, R: BufRead + Seek>(
    reader: &'a mut png::Reader<R>,
    path: &Path,
) -> Result<&'a [u8], AppError> {
    next_optional_row(reader, path)?
        .ok_or_else(|| AppError::InvalidData(format!("PNG ended early: {}", path.display())))
}

fn next_optional_row<'a, R: BufRead + Seek>(
    reader: &'a mut png::Reader<R>,
    path: &Path,
) -> Result<Option<&'a [u8]>, AppError> {
    reader
        .next_row()
        .map(|row| row.map(|value| value.data()))
        .map_err(|source| AppError::Png {
            path: path.to_owned(),
            source,
        })
}

fn gameplay_level(render_level: u8) -> RiverLevel {
    // EU5 uses three consecutive render-palette steps for each gameplay tier.
    // A source-only pixel is still a brook; the thirteenth step is the fifth tier.
    let normalized = render_level.max(1);
    RiverLevel(((normalized - 1) / 3 + 1).min(MAX_RIVER_LEVEL))
}

#[cfg(test)]
mod tests;
