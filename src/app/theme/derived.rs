//! Colours derived from the palette tokens. They are computed on use and never stored, so a
//! host palette change or a fade frame needs no recomputation step.

use iced::{Color, Shadow, Vector};

use super::constants::{
    ERROR_VEIL_ALPHA, HOVER_ACCENT_MIX, LOADING_VEIL_ALPHA, PRESSED_LINE_STRONG_MIX,
    SCRIM_ALPHA_DARK, SCRIM_ALPHA_LIGHT, SCROLLBAR_THUMB_ALPHA, SHADOW_ALPHA_DARK,
    SHADOW_ALPHA_LIGHT, SHADOW_BLUR, SHADOW_OFFSET_Y, SURFACE_2_LINE_MIX, ZEBRA_PAPER_MIX,
};
use super::palette::{Mode, Palette, mix};

/// Scrollbar thumb interaction state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ThumbState {
    /// At rest.
    Idle,
    /// Pointer over the thumb or lane.
    Hovered,
    /// Being dragged.
    Dragged,
}

impl Palette {
    /// Toolbar, grid header and pressed fills.
    #[must_use]
    pub(crate) fn surface_2(&self) -> Color {
        mix(self.surface, self.line, SURFACE_2_LINE_MIX)
    }

    /// Odd grid rows.
    #[must_use]
    pub(crate) fn zebra(&self) -> Color {
        mix(self.surface, self.paper, ZEBRA_PAPER_MIX)
    }

    /// Hovered row or option over a ground.
    #[must_use]
    pub(crate) fn hover(&self, ground: Color) -> Color {
        mix(ground, self.accent_soft, HOVER_ACCENT_MIX)
    }

    /// Pressed button ground.
    #[must_use]
    pub(crate) fn pressed(&self) -> Color {
        mix(self.surface_2(), self.line_strong, PRESSED_LINE_STRONG_MIX)
    }

    /// Scrollbar thumb colour for an interaction state.
    #[must_use]
    pub(crate) fn scrollbar_thumb(&self, state: ThumbState) -> Color {
        match state {
            ThumbState::Idle => self.ink_faint.scale_alpha(SCROLLBAR_THUMB_ALPHA),
            ThumbState::Hovered => self.ink_muted,
            ThumbState::Dragged => self.accent,
        }
    }

    /// Veil behind the loading card.
    #[must_use]
    pub(crate) fn loading_veil(&self) -> Color {
        self.paper.scale_alpha(LOADING_VEIL_ALPHA)
    }

    /// Veil behind the load-error card.
    #[must_use]
    pub(crate) fn error_veil(&self) -> Color {
        self.paper.scale_alpha(ERROR_VEIL_ALPHA)
    }

    /// Black scrim behind drawers and dialogs, by mode.
    #[must_use]
    pub(crate) fn scrim(&self) -> Color {
        let alpha = match self.mode {
            Mode::Light => SCRIM_ALPHA_LIGHT,
            Mode::Dark => SCRIM_ALPHA_DARK,
        };
        Color::BLACK.scale_alpha(alpha)
    }

    /// Shadow under menus, dialogs, cards and drawers.
    #[must_use]
    pub(crate) fn shadow(&self) -> Shadow {
        let alpha = match self.mode {
            Mode::Light => SHADOW_ALPHA_LIGHT,
            Mode::Dark => SHADOW_ALPHA_DARK,
        };
        Shadow {
            color: Color::BLACK.scale_alpha(alpha),
            offset: Vector::new(0.0, SHADOW_OFFSET_Y),
            blur_radius: SHADOW_BLUR,
        }
    }

    /// Text on an accent ground.
    #[must_use]
    pub(crate) const fn on_accent(&self) -> Color {
        self.paper
    }

    /// Food-producing raw materials.
    #[must_use]
    pub(crate) const fn food(&self) -> Color {
        self.ok
    }

    /// Gold and silver raw materials.
    #[must_use]
    pub(crate) const fn precious(&self) -> Color {
        self.accent
    }

    /// River bonus cells.
    #[must_use]
    pub(crate) const fn river_cell(&self) -> Color {
        self.accent_strong
    }
}
