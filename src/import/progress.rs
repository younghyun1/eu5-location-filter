//! Typed import progress shared by the command line and the desktop loader.

/// One step of an import, in the order the importer and the command line run them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImportStage {
    /// Parsing `map_data/location_templates.txt`.
    LocationTemplates,
    /// Parsing `map_data/definitions.txt`.
    Hierarchy,
    /// Parsing `map_data/named_locations/00_default.txt`.
    MapColors,
    /// Parsing `map_data/ports.csv`.
    Ports,
    /// Parsing the river width defines.
    RiverSettings,
    /// Reading `default.map` and the topography, vegetation, climate and modifier definitions.
    StaticMapFactors,
    /// Reading English localization for names and labels.
    Localization,
    /// Scanning `locations.png` and `rivers.png`.
    MapAndRivers,
    /// Computing static population capacity and resolving ports.
    PopulationCapacity,
    /// Building, validating and writing the non-English language packs.
    LanguagePacks,
    /// Everything requested has finished.
    Complete,
}

impl ImportStage {
    /// English text the command line prints for this stage.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::LocationTemplates => "Reading location templates",
            Self::Hierarchy => "Reading hierarchy",
            Self::MapColors => "Reading map colors",
            Self::Ports => "Reading ports",
            Self::RiverSettings => "Reading river settings",
            Self::StaticMapFactors => "Reading static map factors",
            Self::Localization => "Reading English localization",
            Self::MapAndRivers => "Scanning map and rivers",
            Self::PopulationCapacity => "Calculating static population capacity",
            Self::LanguagePacks => "Building language packs",
            Self::Complete => "Import complete",
        }
    }
}

/// Import progress for a terminal or the desktop loader.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ImportProgress {
    /// Current stage.
    pub stage: ImportStage,
    /// Completed work units.
    pub current: u64,
    /// Total work units, or zero when unknown.
    pub total: u64,
}

/// Sends one progress value; a small helper so call sites stay one line each.
pub(super) fn report(
    progress: &mut impl FnMut(ImportProgress),
    stage: ImportStage,
    current: u64,
    total: u64,
) {
    progress(ImportProgress {
        stage,
        current,
        total,
    });
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::ImportStage;

    #[test]
    fn every_stage_has_a_distinct_label() {
        let stages = [
            ImportStage::LocationTemplates,
            ImportStage::Hierarchy,
            ImportStage::MapColors,
            ImportStage::Ports,
            ImportStage::RiverSettings,
            ImportStage::StaticMapFactors,
            ImportStage::Localization,
            ImportStage::MapAndRivers,
            ImportStage::PopulationCapacity,
            ImportStage::LanguagePacks,
            ImportStage::Complete,
        ];
        let labels: HashSet<&str> = stages.iter().map(|stage| stage.label()).collect();
        assert_eq!(labels.len(), stages.len());
        assert!(labels.iter().all(|label| !label.is_empty()));
    }
}
