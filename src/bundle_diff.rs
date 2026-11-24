//! Field-level comparison of two location bundles, keyed by location identifier.
//!
//! Symbols are compared by their resolved text because each bundle owns its own
//! dictionary, so numeric `SymbolId` values are not comparable across bundles.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::model::{Dataset, LocationRecord, SymbolId};

/// Number of example changes retained per field; the report stays bounded for any input.
const MAX_EXAMPLES: usize = 12;

/// Per-field change count with a bounded list of examples.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FieldChanges {
    /// Locations whose value differs between the bundles.
    pub count: usize,
    /// Up to [`MAX_EXAMPLES`] lines of `key: old -> new`.
    pub examples: Vec<String>,
}

/// Result of comparing an older bundle with a newer one.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BundleDiff {
    /// Build identifier of the older bundle.
    pub old_build: u64,
    /// Build identifier of the newer bundle.
    pub new_build: u64,
    /// Location count of the older bundle.
    pub old_count: usize,
    /// Location count of the newer bundle.
    pub new_count: usize,
    /// Identifiers present only in the newer bundle, sorted.
    pub added: Vec<String>,
    /// Identifiers present only in the older bundle, sorted.
    pub removed: Vec<String>,
    /// Changes per field name, in field-name order.
    pub fields: BTreeMap<&'static str, FieldChanges>,
}

impl BundleDiff {
    /// Renders a plain-text report with bounded example lists.
    #[must_use]
    pub fn report(&self) -> String {
        let mut output = String::new();
        let _ = writeln!(
            output,
            "build {} ({} locations) -> build {} ({} locations)",
            self.old_build, self.old_count, self.new_build, self.new_count
        );
        write_keys(&mut output, "added", &self.added);
        write_keys(&mut output, "removed", &self.removed);
        for (field, changes) in &self.fields {
            let _ = writeln!(output, "{field}: {} changed", changes.count);
            for example in &changes.examples {
                let _ = writeln!(output, "  {example}");
            }
        }
        output
    }
}

fn write_keys(output: &mut String, label: &str, keys: &[String]) {
    let _ = writeln!(output, "{label}: {}", keys.len());
    for key in keys.iter().take(MAX_EXAMPLES) {
        let _ = writeln!(output, "  {key}");
    }
}

/// Compares every location shared by both bundles and lists added and removed identifiers.
#[must_use]
pub fn diff(old: &Dataset, new: &Dataset) -> BundleDiff {
    let old_rows = rows_by_key(old);
    let new_rows = rows_by_key(new);
    let mut result = BundleDiff {
        old_build: old.stored.build_id,
        new_build: new.stored.build_id,
        old_count: old.stored.locations.len(),
        new_count: new.stored.locations.len(),
        ..BundleDiff::default()
    };
    for key in new_rows.keys() {
        if !old_rows.contains_key(key) {
            result.added.push((*key).to_owned());
        }
    }
    for (key, old_record) in &old_rows {
        let Some(new_record) = new_rows.get(key) else {
            result.removed.push((*key).to_owned());
            continue;
        };
        let old_fields = fields(old, old_record);
        let new_fields = fields(new, new_record);
        for ((name, before), (_, after)) in old_fields.into_iter().zip(new_fields) {
            if before == after {
                continue;
            }
            let entry = result.fields.entry(name).or_default();
            entry.count += 1;
            if entry.examples.len() < MAX_EXAMPLES {
                entry.examples.push(format!("{key}: {before} -> {after}"));
            }
        }
    }
    result
}

fn rows_by_key(dataset: &Dataset) -> BTreeMap<&str, &LocationRecord> {
    dataset
        .stored
        .locations
        .iter()
        .filter_map(|record| dataset.symbol(record.key).map(|key| (key, record)))
        .collect()
}

fn text(dataset: &Dataset, symbol: Option<SymbolId>) -> String {
    match symbol.and_then(|id| dataset.symbol(id)) {
        Some(value) => value.to_owned(),
        None => "-".to_owned(),
    }
}

/// Field names and rendered values in a fixed order shared by both sides.
fn fields(dataset: &Dataset, record: &LocationRecord) -> Vec<(&'static str, String)> {
    let connected = match record
        .connected_sea
        .and_then(|id| dataset.location(id))
        .and_then(|sea| dataset.symbol(sea.key))
    {
        Some(key) => key.to_owned(),
        None => "-".to_owned(),
    };
    let capacity = record.static_population_capacity;
    vec![
        ("area", text(dataset, Some(record.hierarchy.area))),
        (
            "capacity_equator",
            capacity.map_or_else(|| "-".to_owned(), |value| value.equator.0.to_string()),
        ),
        (
            "capacity_modifier_bp",
            capacity.map_or_else(
                || "-".to_owned(),
                |value| value.modifier_basis_points.to_string(),
            ),
        ),
        (
            "capacity_total",
            capacity.map_or_else(|| "-".to_owned(), |value| value.total.0.to_string()),
        ),
        (
            "capacity_vegetation",
            capacity.map_or_else(|| "-".to_owned(), |value| value.vegetation.0.to_string()),
        ),
        ("climate", text(dataset, record.climate)),
        ("coastal", record.coastal.to_string()),
        ("color", record.color.hex()),
        ("connected_sea", connected),
        ("continent", text(dataset, Some(record.hierarchy.continent))),
        ("culture", text(dataset, record.culture)),
        (
            "harbor_suitability",
            record
                .harbor_suitability
                .map_or_else(|| "-".to_owned(), |value| format!("{value:.4}")),
        ),
        ("kind", record.kind.label().to_owned()),
        ("modifier", text(dataset, record.modifier)),
        (
            "movement_assistance",
            record.movement_assistance.map_or_else(
                || "-".to_owned(),
                |value| format!("{:.4},{:.4}", value[0], value[1]),
            ),
        ),
        ("name", text(dataset, Some(record.name))),
        ("province", text(dataset, Some(record.hierarchy.province))),
        ("raw_material", text(dataset, record.raw_material)),
        ("region", text(dataset, Some(record.hierarchy.region))),
        ("religion", text(dataset, record.religion)),
        (
            "river",
            record.river.map_or_else(
                || "-".to_owned(),
                |river| {
                    format!(
                        "L{} source={} confluence={}",
                        river.level.0, river.has_source, river.has_confluence
                    )
                },
            ),
        ),
        (
            "subcontinent",
            text(dataset, Some(record.hierarchy.subcontinent)),
        ),
        ("topography", text(dataset, Some(record.topography))),
        ("vegetation", text(dataset, record.vegetation)),
    ]
}

#[cfg(test)]
mod tests {
    use super::diff;

    #[test]
    fn reports_changed_fields_and_removed_locations() {
        let loaded = crate::embedded::load();
        assert!(loaded.is_ok());
        let Ok((old, _)) = loaded else { return };
        let unchanged = diff(&old, &old);
        assert!(unchanged.added.is_empty());
        assert!(unchanged.removed.is_empty());
        assert!(unchanged.fields.is_empty());

        let mut new = old.clone();
        let removed = new.stored.locations.pop();
        assert!(removed.is_some());
        let Some(first) = new.stored.locations.first_mut() else {
            return;
        };
        first.coastal = !first.coastal;
        let changed = diff(&old, &new);
        assert_eq!(changed.removed.len(), 1);
        assert!(changed.added.is_empty());
        assert_eq!(
            changed.fields.get("coastal").map(|field| field.count),
            Some(1)
        );
        assert!(changed.report().contains("coastal: 1 changed"));
    }
}
