//! Field-level comparison of two location bundles, keyed by location identifier, and of their
//! game text per language.
//!
//! Symbols are compared by their resolved text because each bundle owns its own
//! dictionary, so numeric `SymbolId` values are not comparable across bundles.

mod text;
#[cfg(test)]
mod tests;
mod values;

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::model::{Dataset, LocationRecord};
pub use text::{TextDiff, diff_text};
use values::fields;

/// Number of example changes retained per field; the report stays bounded for any input.
const MAX_EXAMPLES: usize = 12;

/// Per-field change count with a bounded list of examples.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FieldChanges {
    /// Locations whose value differs between the bundles.
    pub count: usize,
    /// Up to twelve lines of `key: old -> new`.
    pub examples: Vec<String>,
}

impl FieldChanges {
    /// Counts one change and keeps it as an example while fewer than the limit are stored.
    fn record(&mut self, key: &str, before: &str, after: &str) {
        self.count += 1;
        if self.examples.len() < MAX_EXAMPLES {
            self.examples.push(format!("{key}: {before} -> {after}"));
        }
    }
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
            result
                .fields
                .entry(name)
                .or_default()
                .record(key, &before, &after);
        }
    }
    result
}

/// Records by internal identifier, in identifier order so reports are deterministic.
fn rows_by_key(dataset: &Dataset) -> BTreeMap<&str, &LocationRecord> {
    dataset
        .stored
        .locations
        .iter()
        .filter_map(|record| dataset.symbol(record.key).map(|key| (key, record)))
        .collect()
}
