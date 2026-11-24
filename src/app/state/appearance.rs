//! Palette, palette fade, derived iced theme, and the desktop appearance choices.

use iced::time::Instant;

use crate::app::theme::{AccentPreset, Mode, Palette, PaletteFade};

/// Everything that decides colours and motion.
#[derive(Clone, Debug)]
pub(crate) struct Appearance {
    /// Palette drawn this frame; equals `fade.target()` once the fade settles.
    pub(crate) palette: Palette,
    /// Interpolation from the previous palette to the latest target.
    pub(crate) fade: PaletteFade,
    /// iced theme built from `palette` for stock widget defaults and the window background;
    /// rebuilt only when `palette` changes, because building one generates a full palette.
    pub(crate) theme: iced::Theme,
    /// Time of the latest animation frame, used to sample animations in `view`.
    pub(crate) now: Instant,
    /// Desktop mode choice.
    pub(crate) mode: Mode,
    /// Desktop accent choice.
    pub(crate) accent: AccentPreset,
    /// Desktop Animations choice; the browser sets `reduced_motion` from the host instead.
    pub(crate) motion_enabled: bool,
    /// Effective flag: no palette mix and no drawer slide.
    pub(crate) reduced_motion: bool,
}

impl Appearance {
    /// Settled appearance for a starting palette.
    #[must_use]
    pub(crate) fn new(palette: Palette) -> Self {
        Self {
            palette,
            fade: PaletteFade::settled(palette),
            theme: palette.iced_theme(),
            now: Instant::now(),
            mode: palette.mode,
            accent: AccentPreset::Amber,
            motion_enabled: true,
            reduced_motion: false,
        }
    }
}
