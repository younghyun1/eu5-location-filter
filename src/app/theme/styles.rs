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
use crate::app::layout::{BORDER_WIDTH, RADIUS};

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

/// Button style for a role and status.
#[must_use]
pub(crate) fn button(
    palette: Palette,
    kind: ButtonKind,
    status: button_widget::Status,
) -> button_widget::Style {
    let _ = (kind, status);
    button_widget::Style {
        background: Some(Background::Color(palette.surface)),
        text_color: palette.ink,
        border: hairline(palette.line),
        ..button_widget::Style::default()
    }
}

/// Text field style; `invalid` turns the border `danger`.
#[must_use]
pub(crate) fn text_input(
    palette: Palette,
    invalid: bool,
    status: input_widget::Status,
) -> input_widget::Style {
    let _ = (invalid, status);
    input_widget::Style {
        background: Background::Color(palette.surface),
        border: hairline(palette.line),
        icon: palette.ink_muted,
        placeholder: palette.ink_faint,
        value: palette.ink,
        selection: palette.accent_soft,
    }
}

/// Checkbox style for toggles; checklist rows are drawn by the checklist widget itself.
#[must_use]
pub(crate) fn checkbox(
    palette: Palette,
    status: checkbox_widget::Status,
) -> checkbox_widget::Style {
    let _ = status;
    checkbox_widget::Style {
        background: Background::Color(palette.surface),
        icon_color: palette.on_accent(),
        border: hairline(palette.line_strong),
        text_color: Some(palette.ink),
    }
}

/// Scrollable style for the rail, detail, About and menu content.
#[must_use]
pub(crate) fn scrollable(
    palette: Palette,
    status: scrollable_widget::Status,
) -> scrollable_widget::Style {
    let _ = status;
    let rail = scrollable_widget::Rail {
        background: None,
        border: Border::default(),
        scroller: scrollable_widget::Scroller {
            background: Background::Color(palette.scrollbar_thumb(ThumbState::Idle)),
            border: Border::default().rounded(RADIUS),
        },
    };
    scrollable_widget::Style {
        container: container::Style::default(),
        vertical_rail: rail,
        horizontal_rail: rail,
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
