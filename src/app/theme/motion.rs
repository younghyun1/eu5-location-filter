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
        if reduced_motion {
            *self = Self::settled(target);
            return;
        }
        // The host and the frame loop may repeat the palette they already sent; restarting the
        // fade for an identical target would stall it at its first frame.
        if target == self.to {
            return;
        }
        // Start from what is on screen now, so a retarget in mid-fade does not jump.
        let from = self.sample(now);
        self.from = from;
        self.to = target;
        self.progress = Animation::new(false)
            .duration(palette_duration(false))
            .easing(palette_easing())
            .go(true, now);
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
/// x for its parameter, then returns its y. The x control points lie in `0..=1`, so x is
/// monotonic in the parameter and bisection converges. Input is clamped to `0..=1` and NaN maps to 0.
#[must_use]
pub(crate) fn chrome_curve(x: f32) -> f32 {
    if x.is_nan() {
        return 0.0;
    }
    // The end points are exact so that a finished fade is exactly its target.
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let [x1, y1, x2, y2] = PALETTE_EASING_BEZIER;
    let mut low = 0.0_f32;
    let mut high = 1.0_f32;
    for _ in 0..BISECTION_STEPS {
        let middle = (low + high) * 0.5;
        if bezier(x1, x2, middle) < x {
            low = middle;
        } else {
            high = middle;
        }
    }
    bezier(y1, y2, (low + high) * 0.5)
}

/// Bisection steps; 24 halvings of the unit interval are finer than an `f32` can resolve.
const BISECTION_STEPS: u32 = 24;

/// One coordinate of a cubic Bezier with end points 0 and 1 and control points `p1`, `p2`.
fn bezier(p1: f32, p2: f32, t: f32) -> f32 {
    let u = 1.0 - t;
    3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t
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
