//! Loading stage text and progress fractions for the desktop loader. The browser has no staged
//! progress: its decode is one synchronous step.

use crate::app::i18n::Text;
use crate::import::ImportStage;

/// Catalog key for an importer stage. The match is exhaustive, so a new `ImportStage` cannot
/// reach the interface without a catalog entry. `LanguagePacks` shows the language-name text
/// because the catalog has no separate key for it and the desktop rebuild does not run that
/// stage.
#[must_use]
pub(crate) fn import_stage_text(stage: ImportStage) -> Text {
    match stage {
        ImportStage::LocationTemplates => Text::ImportTemplates,
        ImportStage::Hierarchy => Text::ImportHierarchy,
        ImportStage::MapColors => Text::ImportColors,
        ImportStage::Ports => Text::ImportPorts,
        ImportStage::RiverSettings => Text::ImportRiverSettings,
        ImportStage::StaticMapFactors => Text::ImportMapFactors,
        ImportStage::Localization => Text::ImportLocalization,
        ImportStage::MapAndRivers => Text::ImportRivers,
        ImportStage::PopulationCapacity => Text::ImportCapacity,
        ImportStage::LanguagePacks => Text::LoadNames,
        ImportStage::Complete => Text::ImportComplete,
    }
}

/// Completed fraction in `0.0..=1.0`; zero when the total is unknown, one when `current` is
/// past `total`.
#[must_use]
pub(crate) fn fraction(current: u64, total: u64) -> f32 {
    if total == 0 {
        return 0.0;
    }
    (current as f32 / total as f32).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::{fraction, import_stage_text};
    use crate::app::i18n::Text;
    use crate::import::ImportStage;

    #[test]
    fn fraction_handles_unknown_and_overshoot() {
        assert!(fraction(3, 0).abs() < f32::EPSILON);
        assert!(fraction(0, 4).abs() < f32::EPSILON);
        assert!((fraction(1, 4) - 0.25).abs() < f32::EPSILON);
        assert!((fraction(9, 4) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn stages_map_to_their_catalog_keys() {
        assert_eq!(import_stage_text(ImportStage::Hierarchy), Text::ImportHierarchy);
        assert_eq!(import_stage_text(ImportStage::MapAndRivers), Text::ImportRivers);
        assert_eq!(import_stage_text(ImportStage::Complete), Text::ImportComplete);
        assert_eq!(import_stage_text(ImportStage::LanguagePacks), Text::LoadNames);
    }
}
