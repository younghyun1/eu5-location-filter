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

pub(crate) use accent::AccentPreset;
pub(crate) use constants::*;
pub(crate) use derived::ThumbState;
pub(crate) use fonts::{
    MONO, MONO_MEDIUM, SANS, cached_line_height, font_for_text, note_font_registered, sans_for,
};
pub(crate) use glyphs::Icon;
pub(crate) use motion::{PaletteFade, drawer_duration};
#[cfg(test)]
pub(crate) use motion::{chrome_curve, palette_duration};
pub(crate) use palette::{Mode, PALETTE_DARK, Palette};
#[cfg(test)]
pub(crate) use palette::{PALETTE_LIGHT, mix};
#[cfg(test)]
pub(crate) use styles::text_color;
pub(crate) use styles::{ButtonKind, button, checkbox, progress_bar, rule, scrollable, text_input};
pub(crate) use surfaces::{
    Veil, badge, card, detail_block, header, menu, notice, scrim, surface, swatch, toolbar,
    tooltip, veil,
};
