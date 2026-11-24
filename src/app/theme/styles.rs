//! Style functions for stock widgets. Each takes the palette by value so a `move` closure can
//! call it from a widget's `.style(..)` builder:
//! `.style(move |_theme, status| theme::button(palette, ButtonKind::Primary, status))`.

use iced::widget::{
    button as button_widget, checkbox as checkbox_widget, container,
    progress_bar as progress_widget, rule as rule_widget, scrollable as scrollable_widget,
    text as text_widget, text_input as input_widget,
};
use iced::{Background, Border, Color, Shadow};

use super::derived::ThumbState;
use super::palette::Palette;
use crate::app::layout::{BORDER_WIDTH, FOCUS_RING_WIDTH, RADIUS};

/// Button roles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ButtonKind {
    /// Accent ground, paper text: Retry, the selected appearance choice.
    Primary,
    /// Surface ground with a border: Columns, Show details, Close, Rebuild data.
    Secondary,
    /// No ground until hovered: icon buttons, fold buttons, menu items, links.
    Ghost,
}

/// Button style for a role and status. Focus is not a button status in iced: the focus ring is
/// drawn by the `FocusRing` wrapper around the button.
#[must_use]
pub(crate) fn button(
    palette: Palette,
    kind: ButtonKind,
    status: button_widget::Status,
) -> button_widget::Style {
    use button_widget::Status;
    let disabled = matches!(status, Status::Disabled);
    let (background, text_color, border) = match kind {
        ButtonKind::Primary if !disabled => {
            let ground = match status {
                Status::Hovered | Status::Pressed => palette.accent_strong,
                Status::Active | Status::Disabled => palette.accent,
            };
            (
                Some(ground),
                palette.on_accent(),
                Border::default().rounded(RADIUS),
            )
        }
        ButtonKind::Primary | ButtonKind::Secondary => {
            let (ground, text, line) = match status {
                Status::Active => (palette.surface, palette.ink, palette.line_strong),
                Status::Hovered => (palette.surface_2(), palette.ink, palette.line_strong),
                Status::Pressed => (palette.pressed(), palette.ink, palette.line_strong),
                Status::Disabled => (palette.surface, palette.ink_faint, palette.line),
            };
            (Some(ground), text, hairline(line))
        }
        ButtonKind::Ghost => {
            let (ground, text) = match status {
                Status::Active => (None, palette.ink_muted),
                Status::Hovered => (Some(palette.surface_2()), palette.ink),
                Status::Pressed => (Some(palette.pressed()), palette.ink),
                Status::Disabled => (None, palette.ink_faint),
            };
            (ground, text, Border::default().rounded(RADIUS))
        }
    };
    button_widget::Style {
        background: background.map(Background::Color),
        text_color,
        border,
        shadow: Shadow::default(),
        snap: false,
    }
}

/// Text field style; `invalid` turns the border `danger`. Focus widens the border to
/// `FOCUS_RING_WIDTH`; iced draws a border inside the field's bounds, so layout never moves.
#[must_use]
pub(crate) fn text_input(
    palette: Palette,
    invalid: bool,
    status: input_widget::Status,
) -> input_widget::Style {
    use input_widget::Status;
    let focus_color = if invalid {
        palette.danger
    } else {
        palette.accent
    };
    let rest_color = if invalid {
        palette.danger
    } else {
        palette.line_strong
    };
    let border = match status {
        Status::Active => hairline(rest_color),
        Status::Hovered => hairline(if invalid {
            palette.danger
        } else {
            palette.ink_faint
        }),
        Status::Focused { .. } => Border {
            color: focus_color,
            width: FOCUS_RING_WIDTH,
            radius: RADIUS.into(),
        },
        Status::Disabled => hairline(palette.line),
    };
    let disabled = matches!(status, Status::Disabled);
    input_widget::Style {
        background: Background::Color(palette.surface),
        border,
        icon: palette.ink_muted,
        placeholder: palette.ink_faint,
        value: if disabled {
            palette.ink_faint
        } else {
            palette.ink
        },
        selection: palette.accent_soft,
    }
}

/// Checkbox style for toggles; checklist rows are drawn by the checklist widget itself. The
/// active-option outline of a keyboard-focused row belongs to the surrounding focus ring.
#[must_use]
pub(crate) fn checkbox(
    palette: Palette,
    status: checkbox_widget::Status,
) -> checkbox_widget::Style {
    use checkbox_widget::Status;
    let (checked, hovered, disabled) = match status {
        Status::Active { is_checked } => (is_checked, false, false),
        Status::Hovered { is_checked } => (is_checked, true, false),
        Status::Disabled { is_checked } => (is_checked, false, true),
    };
    let (background, border_color) = match (checked, hovered, disabled) {
        (false, _, true) => (palette.surface, palette.line),
        (false, true, false) => (palette.surface, palette.ink_faint),
        (false, false, false) => (palette.surface, palette.line_strong),
        (true, _, true) => (palette.line_strong, palette.line_strong),
        (true, true, false) => (palette.accent_strong, palette.accent_strong),
        (true, false, false) => (palette.accent, palette.accent),
    };
    checkbox_widget::Style {
        background: Background::Color(background),
        icon_color: palette.on_accent(),
        border: hairline(border_color),
        text_color: Some(if disabled {
            palette.ink_faint
        } else {
            palette.ink
        }),
    }
}

/// Scrollable style for the rail, detail, About and menu content. Each rail's thumb follows its
/// own interaction: resting, pointer over that scrollbar, or being dragged.
#[must_use]
pub(crate) fn scrollable(
    palette: Palette,
    status: scrollable_widget::Status,
) -> scrollable_widget::Style {
    use scrollable_widget::Status;
    let (vertical, horizontal) = match status {
        Status::Active { .. } => (ThumbState::Idle, ThumbState::Idle),
        Status::Hovered {
            is_horizontal_scrollbar_hovered,
            is_vertical_scrollbar_hovered,
            ..
        } => (
            if is_vertical_scrollbar_hovered {
                ThumbState::Hovered
            } else {
                ThumbState::Idle
            },
            if is_horizontal_scrollbar_hovered {
                ThumbState::Hovered
            } else {
                ThumbState::Idle
            },
        ),
        Status::Dragged {
            is_horizontal_scrollbar_dragged,
            is_vertical_scrollbar_dragged,
            ..
        } => (
            if is_vertical_scrollbar_dragged {
                ThumbState::Dragged
            } else {
                ThumbState::Idle
            },
            if is_horizontal_scrollbar_dragged {
                ThumbState::Dragged
            } else {
                ThumbState::Idle
            },
        ),
    };
    let rail = |state: ThumbState| scrollable_widget::Rail {
        background: None,
        border: Border::default(),
        scroller: scrollable_widget::Scroller {
            background: Background::Color(palette.scrollbar_thumb(state)),
            border: Border::default().rounded(RADIUS),
        },
    };
    scrollable_widget::Style {
        container: container::Style::default(),
        vertical_rail: rail(vertical),
        horizontal_rail: rail(horizontal),
        gap: None,
        auto_scroll: scrollable_widget::AutoScroll {
            background: Background::Color(palette.surface),
            border: hairline(palette.line_strong),
            shadow: Shadow::default(),
            icon: palette.ink,
        },
    }
}

/// Loading progress bar: `line` track, `accent` fill.
#[must_use]
pub(crate) fn progress_bar(palette: Palette) -> progress_widget::Style {
    progress_widget::Style {
        background: Background::Color(palette.line),
        bar: Background::Color(palette.accent),
        border: Border::default(),
    }
}

/// Divider rule; `strong` uses `line_strong`.
#[must_use]
pub(crate) fn rule(palette: Palette, strong: bool) -> rule_widget::Style {
    rule_widget::Style {
        color: if strong {
            palette.line_strong
        } else {
            palette.line
        },
        radius: iced::border::Radius::from(0.0_f32),
        fill_mode: rule_widget::FillMode::Full,
        snap: true,
    }
}

/// Text style with an explicit colour.
#[must_use]
pub(crate) fn text_color(color: Color) -> text_widget::Style {
    text_widget::Style { color: Some(color) }
}

/// Hairline border with the shared corner radius.
fn hairline(color: Color) -> Border {
    Border {
        color,
        width: BORDER_WIDTH,
        radius: RADIUS.into(),
    }
}
