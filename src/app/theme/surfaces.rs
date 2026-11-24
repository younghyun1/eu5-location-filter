//! Container styles for panes, cards, menus, veils, scrims, badges and notices.

use iced::widget::container::Style;
use iced::{Background, Border, Color};

use super::palette::Palette;
use crate::app::layout::{BORDER_WIDTH, RADIUS, RADIUS_PILL};

/// Which veil covers the window.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Veil {
    /// Behind the loading card.
    Loading,
    /// Behind the load-error card.
    Error,
}

/// Pane ground: rail, detail, header.
#[must_use]
pub(crate) fn surface(palette: Palette) -> Style {
    filled(palette.surface, palette.ink)
}

/// Header bar ground.
#[must_use]
pub(crate) fn header(palette: Palette) -> Style {
    filled(palette.surface, palette.ink)
}

/// Results toolbar and other `surface_2` strips.
#[must_use]
pub(crate) fn toolbar(palette: Palette) -> Style {
    filled(palette.surface_2(), palette.ink)
}

/// Floating card over a veil: loading and error cards, About dialog.
#[must_use]
pub(crate) fn card(palette: Palette) -> Style {
    Style {
        shadow: palette.shadow(),
        border: bordered(palette.line_strong, RADIUS),
        ..filled(palette.surface, palette.ink)
    }
}

/// Popover menu frame.
#[must_use]
pub(crate) fn menu(palette: Palette) -> Style {
    card(palette)
}

/// Window-covering veil.
#[must_use]
pub(crate) fn veil(palette: Palette, kind: Veil) -> Style {
    let color = match kind {
        Veil::Loading => palette.loading_veil(),
        Veil::Error => palette.error_veil(),
    };
    filled(color, palette.ink)
}

/// Scrim behind drawers and dialogs; `opacity` in `0..=1` follows the drawer animation.
#[must_use]
pub(crate) fn scrim(palette: Palette, opacity: f32) -> Style {
    filled(palette.scrim().scale_alpha(opacity.clamp(0.0, 1.0)), palette.ink)
}

/// Count badge pill.
#[must_use]
pub(crate) fn badge(palette: Palette) -> Style {
    Style {
        border: bordered(palette.accent, RADIUS_PILL),
        ..filled(palette.accent_soft, palette.ink)
    }
}

/// Notice bar under the header.
#[must_use]
pub(crate) fn notice(palette: Palette) -> Style {
    Style {
        border: bordered(palette.line, 0.0),
        ..filled(palette.surface, palette.ink)
    }
}

/// Tooltip body.
#[must_use]
pub(crate) fn tooltip(palette: Palette) -> Style {
    card(palette)
}

/// Technical error detail block.
#[must_use]
pub(crate) fn detail_block(palette: Palette) -> Style {
    Style {
        border: bordered(palette.line, RADIUS),
        ..filled(palette.surface_2(), palette.ink_muted)
    }
}

/// Colour swatch with a `line_strong` border.
#[must_use]
pub(crate) fn swatch(palette: Palette, color: Color) -> Style {
    Style {
        border: bordered(palette.line_strong, RADIUS),
        ..filled(color, palette.ink)
    }
}

fn filled(ground: Color, text: Color) -> Style {
    Style {
        text_color: Some(text),
        background: Some(Background::Color(ground)),
        ..Style::default()
    }
}

fn bordered(color: Color, radius: f32) -> Border {
    Border {
        color,
        width: BORDER_WIDTH,
        radius: radius.into(),
    }
}
