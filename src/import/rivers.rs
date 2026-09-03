//! Row-streamed location and river image pairing.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Seek};
use std::path::Path;

use png::{BitDepth, ColorType, Decoder};

use crate::AppError;
use crate::model::{LocationId, MapColor, RiverData, RiverLevel, RiverWidthMetadata};

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

/// Result of the paired image pass.
pub(super) struct RiverScan {
    pub values: Vec<Option<RiverData>>,
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
    let mut unknown_pixels = 0_u64;
    for _ in 0..height {
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
        for (rgb, palette) in location_pixels.iter().zip(river_row.iter().copied()) {
            if palette == 0 || palette > widths.level_count.saturating_add(2) {
                continue;
            }
            let color =
                MapColor((u32::from(rgb[0]) << 16) | (u32::from(rgb[1]) << 8) | u32::from(rgb[2]));
            let Some(location_id) = by_color.get(&color) else {
                unknown_pixels = unknown_pixels.saturating_add(1);
                continue;
            };
            let Ok(index) = usize::try_from(location_id.0) else {
                return Err(AppError::InvalidData(
                    "location index does not fit this platform".to_owned(),
                ));
            };
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
                level: RiverLevel(value.level),
                has_source: value.source,
                has_confluence: value.confluence,
                rendered_width: rendered_width(value.level, widths),
            })
        })
        .collect();
    Ok(RiverScan {
        values,
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

fn rendered_width(level: u8, widths: RiverWidthMetadata) -> f32 {
    if level == 0 {
        return 0.0;
    }
    if widths.level_count == 1 {
        return widths.width_min;
    }
    let fraction = f32::from(level - 1) / f32::from(widths.level_count - 1);
    widths.width_min + (widths.width_max - widths.width_min) * fraction
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::io::Cursor;
    use std::path::Path;

    use png::{BitDepth, ColorType, Encoder};

    use super::scan_readers;
    use crate::model::{LocationId, MapColor, RiverWidthMetadata};

    #[test]
    fn streams_sources_confluences_levels_and_unknown_colors() {
        let locations = encode(
            5,
            1,
            ColorType::Rgb,
            &[0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 9, 9, 9],
        );
        let rivers = encode(5, 1, ColorType::Indexed, &[1, 2, 3, 5, 4]);
        let mut colors = HashMap::new();
        colors.insert(MapColor(1), LocationId(0));
        let widths = RiverWidthMetadata {
            level_count: 3,
            width_min: 1.0,
            width_max: 3.0,
        };
        let scanned = scan_readers(
            Cursor::new(locations),
            Cursor::new(rivers),
            Path::new("locations.png"),
            Path::new("rivers.png"),
            &colors,
            1,
            widths,
            Some((5, 1)),
        );
        assert!(scanned.is_ok());
        let Ok(scanned) = scanned else { return };
        let Some(Some(river)) = scanned.values.first() else {
            return;
        };
        assert!(river.has_source);
        assert!(river.has_confluence);
        assert_eq!(river.level.0, 3);
        assert_eq!(scanned.unknown_pixels, 1);
    }

    #[test]
    fn rejects_dimension_mismatch() {
        let locations = encode(1, 1, ColorType::Rgb, &[0, 0, 1]);
        let rivers = encode(2, 1, ColorType::Indexed, &[1, 1]);
        let result = scan_readers(
            Cursor::new(locations),
            Cursor::new(rivers),
            Path::new("locations.png"),
            Path::new("rivers.png"),
            &HashMap::new(),
            0,
            RiverWidthMetadata {
                level_count: 1,
                width_min: 1.0,
                width_max: 1.0,
            },
            None,
        );
        assert!(result.is_err());
    }

    fn encode(width: u32, height: u32, color: ColorType, data: &[u8]) -> Vec<u8> {
        let mut output = Vec::new();
        let mut encoder = Encoder::new(&mut output, width, height);
        encoder.set_color(color);
        encoder.set_depth(BitDepth::Eight);
        if color == ColorType::Indexed {
            encoder.set_palette(vec![0; 256 * 3]);
        }
        {
            let mut writer = match encoder.write_header() {
                Ok(writer) => writer,
                Err(_) => return Vec::new(),
            };
            if writer.write_image_data(data).is_err() {
                return Vec::new();
            }
        }
        output
    }
}
