//! Site palette, derived colours, fonts, glyphs, motion, and the style functions every pane uses.

mod accent;
mod constants;
mod derived;
mod fonts;
mod glyphs;
mod motion;
mod palette;
mod styles;
mod surfaces;
#[cfg(test)]
mod tests;

pub(crate) use constants::*;
pub(crate) use fonts::{MONO, MONO_MEDIUM, SANS, SANS_MEDIUM, SANS_SEMIBOLD, sans_for};
pub(crate) use glyphs::Icon;
pub(crate) use motion::{PaletteFade, chrome_curve, drawer_duration, palette_duration, palette_easing};
pub(crate) use accent::AccentPreset;
pub(crate) use derived::ThumbState;
pub(crate) use palette::{Mode, PALETTE_DARK, PALETTE_LIGHT, Palette, mix};
pub(crate) use styles::{ButtonKind, button, checkbox, progress_bar, rule, scrollable, text_color, text_input};
pub(crate) use surfaces::{Veil, badge, card, detail_block, header, menu, notice, scrim, surface, swatch, toolbar, tooltip, veil};
