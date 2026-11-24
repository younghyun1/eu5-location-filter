//! Loading phases shown by the loading and error overlays.

use std::sync::Arc;

use crate::AppError;
use crate::app::i18n::Text;

/// Where the application is in getting its dataset ready.
#[derive(Clone, Debug)]
pub(crate) enum Phase {
    /// Nothing loaded yet; the browser waits here for its first painted frame.
    AwaitingPaint,
    /// A load is running; the loading card shows this progress.
    Loading(LoadProgress),
    /// The last load failed; the error card shows the failure.
    Failed(LoadFailure),
    /// Dataset installed and interactive.
    Ready,
}

/// Progress of a running load.
#[derive(Clone, Copy, Debug)]
pub(crate) struct LoadProgress {
    /// Catalog text of the current stage (`load.*` or `import.*`).
    pub(crate) stage: Text,
    /// Completed fraction in `0.0..=1.0`; the bar never animates on its own.
    pub(crate) fraction: f32,
}

/// A failed load.
#[derive(Clone, Debug)]
pub(crate) struct LoadFailure {
    /// The error; its variant picks the message family and its `Display` text is the detail.
    pub(crate) error: Arc<AppError>,
}
