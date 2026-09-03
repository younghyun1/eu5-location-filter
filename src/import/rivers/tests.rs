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
