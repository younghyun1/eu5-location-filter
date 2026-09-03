use super::{LocationKind, MapColor};

#[test]
fn color_accepts_omitted_leading_zeroes() {
    assert_eq!(MapColor::parse("1e05").ok(), Some(MapColor(0x001e05)));
    assert!(MapColor::parse("xyz").is_err());
    assert!(MapColor::parse("1234567").is_err());
}

#[test]
fn wasteland_precedes_other_kind_rules() {
    assert_eq!(
        LocationKind::from_topography("ocean_wasteland"),
        LocationKind::Impassable
    );
    assert_eq!(
        LocationKind::from_topography("coastal_ocean"),
        LocationKind::Sea
    );
    assert_eq!(
        LocationKind::from_topography("salt_pans"),
        LocationKind::Impassable
    );
    assert_eq!(
        LocationKind::from_topography("future_biome"),
        LocationKind::Unknown
    );
}
