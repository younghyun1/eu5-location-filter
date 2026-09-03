use super::{closeness_to_equator, direct_effect, nested_effect, whole_people};
use crate::parser::parse_document;

#[test]
fn latitude_factor_uses_map_equator_coordinate() {
    let kotor_port_y = 5_790.0;
    let contribution = 10_000.0 * closeness_to_equator(kotor_port_y, 3_340.0);
    assert!((contribution - 2_664.67).abs() < 0.01);
    assert_eq!(closeness_to_equator(7_000.0, 3_340.0), 0.0);
}

#[test]
fn reads_additive_and_percentage_static_effects() {
    let parsed = parse_document(
        "test",
        b"woods={location_modifier={local_population_capacity=50}} coastal={local_population_capacity_modifier=.25}",
    );
    assert!(parsed.is_ok());
    let Ok(entries) = parsed else { return };
    let woods = entries.first().and_then(|entry| match entry {
        crate::parser::Entry::Assignment(_, crate::parser::Value::Block(fields)) => {
            nested_effect(fields, "location_modifier", "local_population_capacity")
                .ok()
                .flatten()
        }
        _ => None,
    });
    assert_eq!(woods, Some(50.0));
    assert_eq!(
        direct_effect(&entries, "coastal", "local_population_capacity_modifier").ok(),
        Some(0.25)
    );
}

#[test]
fn whole_people_matches_game_truncation() {
    assert_eq!(whole_people(2_661.9, "test").ok(), Some(2_661));
    assert!(whole_people(-1.0, "test").is_err());
}
