//! Chrome curve, palette fade and reduced-motion behaviour.

use iced::time::{Duration, Instant};

use super::common::{close, colors_close};
use crate::app::theme::{
    DRAWER_SLIDE_MS, PALETTE_DARK, PALETTE_LIGHT, PALETTE_MIX_MS, PaletteFade, chrome_curve,
    drawer_duration, palette_duration,
};

/// Time after the start by which a fade has certainly finished.
const WELL_AFTER: Duration = Duration::from_millis(500);

#[test]
fn curve_starts_at_zero_and_ends_at_one() {
    assert!(close(chrome_curve(0.0), 0.0));
    assert!(close(chrome_curve(1.0), 1.0));
}

#[test]
fn curve_clamps_and_survives_non_finite_input() {
    assert!(close(chrome_curve(-5.0), 0.0));
    assert!(close(chrome_curve(5.0), 1.0));
    assert!(close(chrome_curve(f32::NAN), 0.0));
    assert!(close(chrome_curve(f32::NEG_INFINITY), 0.0));
}

#[test]
fn curve_is_monotonic_and_stays_in_the_unit_range() {
    let mut previous = 0.0_f32;
    for step in 0..=200_u16 {
        let value = chrome_curve(f32::from(step) / 200.0);
        assert!((0.0..=1.0).contains(&value), "step {step}: {value}");
        assert!(
            value + 1e-5 >= previous,
            "step {step}: {value} < {previous}"
        );
        previous = value;
    }
}

#[test]
fn curve_decelerates_like_the_site_easing() {
    // cubic-bezier(0.4, 0, 0.2, 1) is an ease-out shape: past the diagonal at the midpoint.
    let midpoint = chrome_curve(0.5);
    assert!(midpoint > 0.6 && midpoint < 0.85, "midpoint {midpoint}");
    assert!(chrome_curve(0.25) < chrome_curve(0.5));
    assert!(chrome_curve(0.75) > 0.9);
}

#[test]
fn durations_are_zero_with_reduced_motion() {
    assert_eq!(palette_duration(true), Duration::ZERO);
    assert_eq!(drawer_duration(true), Duration::ZERO);
    assert_eq!(
        palette_duration(false),
        Duration::from_millis(PALETTE_MIX_MS)
    );
    assert_eq!(
        drawer_duration(false),
        Duration::from_millis(DRAWER_SLIDE_MS)
    );
}

#[test]
fn settled_fade_shows_its_palette_and_needs_no_frames() {
    let now = Instant::now();
    let fade = PaletteFade::settled(PALETTE_DARK);
    assert_eq!(fade.sample(now), PALETTE_DARK);
    assert_eq!(fade.target(), PALETTE_DARK);
    assert!(!fade.is_animating(now));
}

#[test]
fn retarget_starts_from_the_old_palette_and_ends_on_the_new_one() {
    let start = Instant::now();
    let mut fade = PaletteFade::settled(PALETTE_DARK);
    fade.retarget(PALETTE_LIGHT, start, false);
    assert_eq!(fade.target(), PALETTE_LIGHT);
    assert!(fade.is_animating(start));
    assert!(colors_close(fade.sample(start).paper, PALETTE_DARK.paper));
    let after = start + WELL_AFTER;
    assert!(!fade.is_animating(after));
    assert_eq!(fade.sample(after), PALETTE_LIGHT);
}

#[test]
fn a_running_fade_is_between_its_two_palettes() {
    let start = Instant::now();
    let mut fade = PaletteFade::settled(PALETTE_DARK);
    fade.retarget(PALETTE_LIGHT, start, false);
    let middle = fade.sample(start + Duration::from_millis(PALETTE_MIX_MS / 2));
    assert!(middle.paper.r > PALETTE_DARK.paper.r);
    assert!(middle.paper.r < PALETTE_LIGHT.paper.r);
    assert!(middle.ink.r < PALETTE_DARK.ink.r);
    assert!(middle.ink.r > PALETTE_LIGHT.ink.r);
}

#[test]
fn reduced_motion_jumps_straight_to_the_target() {
    let now = Instant::now();
    let mut fade = PaletteFade::settled(PALETTE_DARK);
    fade.retarget(PALETTE_LIGHT, now, true);
    assert_eq!(fade.sample(now), PALETTE_LIGHT);
    assert!(!fade.is_animating(now));
    assert_eq!(fade.target(), PALETTE_LIGHT);
}

#[test]
fn reduced_motion_cancels_a_running_fade() {
    let start = Instant::now();
    let mut fade = PaletteFade::settled(PALETTE_DARK);
    fade.retarget(PALETTE_LIGHT, start, false);
    fade.retarget(PALETTE_DARK, start, true);
    assert_eq!(fade.sample(start), PALETTE_DARK);
    assert!(!fade.is_animating(start));
}

#[test]
fn repeating_the_same_target_does_not_restart_the_fade() {
    let start = Instant::now();
    let mut fade = PaletteFade::settled(PALETTE_DARK);
    fade.retarget(PALETTE_LIGHT, start, false);
    // A repeated host message arrives after the fade would have ended.
    fade.retarget(PALETTE_LIGHT, start + WELL_AFTER, false);
    assert!(!fade.is_animating(start + WELL_AFTER));
    assert_eq!(fade.sample(start + WELL_AFTER), PALETTE_LIGHT);
}

#[test]
fn retargeting_mid_fade_continues_from_the_visible_palette() {
    let start = Instant::now();
    let mut fade = PaletteFade::settled(PALETTE_DARK);
    fade.retarget(PALETTE_LIGHT, start, false);
    let moment = start + Duration::from_millis(PALETTE_MIX_MS / 2);
    let shown = fade.sample(moment);
    fade.retarget(PALETTE_DARK, moment, false);
    assert!(colors_close(fade.sample(moment).paper, shown.paper));
    assert!(colors_close(fade.sample(moment).ink, shown.ink));
    assert_eq!(fade.target(), PALETTE_DARK);
    assert_eq!(fade.sample(moment + WELL_AFTER), PALETTE_DARK);
}
