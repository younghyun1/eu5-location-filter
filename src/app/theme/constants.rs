//! Typography, colour-mix factors, alphas and motion timings. Values match the interface layout
//! document; change both together.

/// Family name of the embedded sans faces.
pub(crate) const FONT_SANS_FAMILY: &str = "IBM Plex Sans";
/// Family name of the embedded mono faces.
pub(crate) const FONT_MONO_FAMILY: &str = "IBM Plex Mono";

/// Header brand, Mono Medium.
pub(crate) const TEXT_BRAND: f32 = 15.0;
/// Cells, options, buttons, field values, detail values; Sans Regular.
pub(crate) const TEXT_BODY: f32 = 13.0;
/// Text field size in the narrow layout, matching the site's mobile inputs.
pub(crate) const TEXT_INPUT_NARROW: f32 = 16.0;
/// Facet titles, column headers, detail labels, hints, inline errors; Sans.
pub(crate) const TEXT_SMALL: f32 = 12.0;
/// Result count, build text, identifiers, numbers, hex and RGB; Mono Regular.
pub(crate) const TEXT_META: f32 = 12.0;
/// Rail section titles, pane captions, legend, badges; Mono Medium.
pub(crate) const TEXT_CAPTION: f32 = 11.0;
/// Loading stage and empty-state titles; Sans SemiBold.
pub(crate) const TEXT_TITLE: f32 = 16.0;
/// Detail location name, About and error titles; Sans SemiBold.
pub(crate) const TEXT_HEADING: f32 = 20.0;
/// Relative line height for single-line text, iced's default.
pub(crate) const LINE_HEIGHT: f32 = 1.3;
/// Relative line height for wrapping text: detail values, About text, error messages.
pub(crate) const LINE_HEIGHT_PROSE: f32 = 1.45;

/// `surface_2 = mix(surface, line, t)`; reproduces the site's surface-2 in both modes.
pub(crate) const SURFACE_2_LINE_MIX: f32 = 0.5;
/// Odd grid rows are `mix(surface, paper, t)`.
pub(crate) const ZEBRA_PAPER_MIX: f32 = 0.5;
/// A hovered row or option is `mix(ground, accent_soft, t)`.
pub(crate) const HOVER_ACCENT_MIX: f32 = 0.5;
/// A pressed button is `mix(surface_2, line_strong, t)`.
pub(crate) const PRESSED_LINE_STRONG_MIX: f32 = 0.5;
/// Alpha applied to `ink_faint` for a resting scrollbar thumb.
pub(crate) const SCROLLBAR_THUMB_ALPHA: f32 = 0.7;
/// Alpha of the paper veil behind the loading card.
pub(crate) const LOADING_VEIL_ALPHA: f32 = 0.85;
/// Alpha of the paper veil behind the load-error card.
pub(crate) const ERROR_VEIL_ALPHA: f32 = 0.94;
/// Alpha of the black scrim behind drawers and dialogs in light mode.
pub(crate) const SCRIM_ALPHA_LIGHT: f32 = 0.32;
/// Alpha of the black scrim in dark mode.
pub(crate) const SCRIM_ALPHA_DARK: f32 = 0.6;
/// Alpha of the black shadow under menus, dialogs and drawers in light mode.
pub(crate) const SHADOW_ALPHA_LIGHT: f32 = 0.16;
/// Alpha of the same shadow in dark mode.
pub(crate) const SHADOW_ALPHA_DARK: f32 = 0.5;
/// Shadow blur radius.
pub(crate) const SHADOW_BLUR: f32 = 24.0;
/// Shadow downward offset.
pub(crate) const SHADOW_OFFSET_Y: f32 = 8.0;

/// Palette interpolation after a mode or token change, in milliseconds.
pub(crate) const PALETTE_MIX_MS: u64 = 90;
/// Drawer slide and scrim fade, in milliseconds.
pub(crate) const DRAWER_SLIDE_MS: u64 = 160;
/// Control points x1, y1, x2, y2 of the palette mix curve, the site's chrome easing.
pub(crate) const PALETTE_EASING_BEZIER: [f32; 4] = [0.4, 0.0, 0.2, 1.0];
