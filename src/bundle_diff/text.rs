//! Game text comparison for one language: location names by location key, and labels by
//! identifier and field.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use super::{FieldChanges, rows_by_key};
use crate::lang_pack::{GameLanguage, LabelField, Labels};

/// Name and label changes of one language between two bundles.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextDiff {
    /// Compared language.
    pub language: GameLanguage,
    /// Whether the older side had a pack for this language; always `false` for English.
    pub old_pack: bool,
    /// Whether the newer side had a pack for this language; always `false` for English.
    pub new_pack: bool,
    /// Locations, shared by both bundles, whose displayed name differs.
    pub names: FieldChanges,
    /// Per `LabelField::key`: identifiers, shared by both bundles, whose label differs.
    pub labels: BTreeMap<&'static str, FieldChanges>,
}

impl TextDiff {
    /// Renders a plain-text section headed `## <code>`, listing only label fields that changed.
    #[must_use]
    pub fn report(&self) -> String {
        let mut output = String::new();
        let _ = writeln!(output, "## {}", self.language.code());
        let _ = writeln!(
            output,
            "packs: old {}, new {}",
            presence(self.language, self.old_pack),
            presence(self.language, self.new_pack)
        );
        write_changes(&mut output, "names", &self.names);
        for (field, changes) in &self.labels {
            write_changes(&mut output, &format!("labels.{field}"), changes);
        }
        output
    }
}

fn presence(language: GameLanguage, present: bool) -> &'static str {
    match (language.is_embedded(), present) {
        (true, _) => "embedded",
        (false, true) => "present",
        (false, false) => "missing",
    }
}

fn write_changes(output: &mut String, label: &str, changes: &FieldChanges) {
    let _ = writeln!(output, "{label}: {} changed", changes.count);
    for example in &changes.examples {
        let _ = writeln!(output, "  {example}");
    }
}

/// Compares the text both views display for `language`.
///
/// Each side shows its pack's text where it has one and English otherwise, so a pack missing
/// on one side shows up as changed text next to `packs: ... missing`.
#[must_use]
pub fn diff_text(language: GameLanguage, old: Labels<'_>, new: Labels<'_>) -> TextDiff {
    let old_rows = rows_by_key(old.dataset());
    let new_rows = rows_by_key(new.dataset());
    let mut names = FieldChanges::default();
    for (key, old_record) in &old_rows {
        let Some(new_record) = new_rows.get(key) else {
            continue;
        };
        let before = old.name(old_record.id).unwrap_or("-");
        let after = new.name(new_record.id).unwrap_or("-");
        if before != after {
            names.record(key, before, after);
        }
    }
    let old_labels = labels_by_identifier(old);
    let new_labels = labels_by_identifier(new);
    let mut labels: BTreeMap<&'static str, FieldChanges> = BTreeMap::new();
    for ((field, identifier), before) in &old_labels {
        let Some(after) = new_labels.get(&(*field, *identifier)) else {
            continue;
        };
        if before != after {
            labels.entry(*field).or_default().record(identifier, before, after);
        }
    }
    TextDiff {
        language,
        old_pack: has_pack(language, old),
        new_pack: has_pack(language, new),
        names,
        labels,
    }
}

fn has_pack(language: GameLanguage, labels: Labels<'_>) -> bool {
    !language.is_embedded() && labels.language() == language
}

/// Displayed label per (field key, identifier), for every symbol a record uses in that field.
fn labels_by_identifier<'a>(labels: Labels<'a>) -> BTreeMap<(&'static str, &'a str), &'a str> {
    let dataset = labels.dataset();
    let mut values = BTreeMap::new();
    for record in &dataset.stored.locations {
        for field in LabelField::ALL {
            let Some(symbol) = field.symbol(record) else {
                continue;
            };
            let Some(identifier) = dataset.symbol(symbol) else {
                continue;
            };
            values
                .entry((field.key(), identifier))
                .or_insert_with(|| labels.label(symbol).unwrap_or("-"));
        }
    }
    values
}
