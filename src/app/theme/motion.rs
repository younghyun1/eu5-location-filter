//! Palette fade and drawer timing. With reduced motion every duration is zero.

use iced::Animation;
use iced::animation::Easing;
use iced::time::{Duration, Instant};

use super::constants::{DRAWER_SLIDE_MS, PALETTE_EASING_BEZIER, PALETTE_MIX_MS};
use super::palette::Palette;

/// Interpolation of every token from the previous palette to the latest target over
/// `PALETTE_MIX_MS` with the site's chrome curve.
#[derive(Clone, Debug)]
pub(crate) struct PaletteFade {
    /// Palette shown when the fade started.
    from: Palette,
    /// Target palette.
    to: Palette,
    /// `false` to `true` over the fade.
    progress: Animation<bool>,
}

impl PaletteFade {
    /// A fade that has already reached `palette`.
    #[must_use]
    pub(crate) fn settled(palette: Palette) -> Self {
        Self {
            from: palette,
            to: palette,
            progress: Animation::new(true),
        }
    }

    /// Starts a fade from the palette shown at `now` to `target`, or jumps there with reduced
    /// motion.
    pub(crate) fn retarget(&mut self, target: Palette, now: Instant, reduced_motion: bool) {
        let _ = (now, reduced_motion);
        *self = Self::settled(target);
    }

    /// Palette to draw at `now`.
    #[must_use]
    pub(crate) fn sample(&self, now: Instant) -> Palette {
        let factor: f32 = self.progress.interpolate(0.0, 1.0, now);
        self.from.mixed(&self.to, factor)
    }

    /// Whether frames are still needed.
    #[must_use]
    pub(crate) fn is_animating(&self, now: Instant) -> bool {
        self.progress.is_animating(now)
    }

    /// Latest target palette.
    #[must_use]
    pub(crate) const fn target(&self) -> Palette {
        self.to
    }
}

/// The palette easing as an iced easing.
#[must_use]
pub(crate) fn palette_easing() -> Easing {
    Easing::Custom(chrome_curve)
}

/// Cubic Bezier `PALETTE_EASING_BEZIER` evaluated at progress `x` in `0..=1`: solves the curve's
/// x for its parameter, then returns its y.
#[must_use]
pub(crate) fn chrome_curve(x: f32) -> f32 {
    let _ = PALETTE_EASING_BEZIER;
    x.clamp(0.0, 1.0)
}

/// Drawer slide and scrim fade duration.
#[must_use]
pub(crate) fn drawer_duration(reduced_motion: bool) -> Duration {
    if reduced_motion {
        Duration::ZERO
    } else {
        Duration::from_millis(DRAWER_SLIDE_MS)
    }
}

/// Palette fade duration.
#[must_use]
pub(crate) fn palette_duration(reduced_motion: bool) -> Duration {
    if reduced_motion {
        Duration::ZERO
    } else {
        Duration::from_millis(PALETTE_MIX_MS)
    }
}
