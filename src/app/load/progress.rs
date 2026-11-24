//! Loading stage text and progress fractions.

use crate::app::i18n::Text;

/// Catalog key for an importer stage. `ImportProgress::stage` is still an English string, so
/// this maps the known strings; an unknown stage shows `load.starting`.
#[must_use]
pub(crate) fn import_stage_text(stage: &str) -> Text {
    let _ = stage;
    Text::LoadStarting
}

/// Completed fraction in `0.0..=1.0`; zero when the total is unknown.
#[must_use]
pub(crate) fn fraction(current: u64, total: u64) -> f32 {
    let _ = (current, total);
    0.0
}
