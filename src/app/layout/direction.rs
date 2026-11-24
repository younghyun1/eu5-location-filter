//! Layout direction and the helpers that mirror geometry for right-to-left locales. Direction
//! changes geometry only; colours and metrics stay the same.

use iced::advanced::text::Alignment as TextAlignment;
use iced::alignment::Horizontal;
use iced::widget::{Row, Text, container};
use iced::{Element, Length};

/// Inline direction of the interface locale.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub(crate) enum Direction {
    /// Left to right.
    #[default]
    Ltr,
    /// Right to left: `ar`, `fa-IR`, `ur-PK`.
    Rtl,
}

impl Direction {
    /// Whether the layout is mirrored.
    #[must_use]
    pub(crate) const fn is_rtl(self) -> bool {
        matches!(self, Self::Rtl)
    }

    /// Screen alignment of the inline start.
    #[must_use]
    pub(crate) const fn start(self) -> Horizontal {
        match self {
            Self::Ltr => Horizontal::Left,
            Self::Rtl => Horizontal::Right,
        }
    }

    /// Screen alignment of the inline end.
    #[must_use]
    pub(crate) const fn end(self) -> Horizontal {
        match self {
            Self::Ltr => Horizontal::Right,
            Self::Rtl => Horizontal::Left,
        }
    }

    /// Text alignment of the inline start. Right-to-left returns `TextAlignment::Default`:
    /// iced 0.14 relayouts a single right-to-left line to its own width only for that
    /// alignment, and draws it displaced under an explicit one. A text that must also sit at
    /// the inline start of a wider box goes through [`anchored_text`].
    #[must_use]
    pub(crate) const fn text_start(self) -> TextAlignment {
        match self {
            Self::Ltr => TextAlignment::Left,
            Self::Rtl => TextAlignment::Default,
        }
    }

    /// Converts a screen x delta into an inline delta (positive toward the inline end).
    #[must_use]
    pub(crate) const fn to_inline(self, screen_dx: f32) -> f32 {
        match self {
            Self::Ltr => screen_dx,
            Self::Rtl => -screen_dx,
        }
    }
}

/// Places a text at `side` of the full available width.
///
/// The text keeps its natural width and default alignment, which is the only combination iced
/// 0.14 lays out correctly for a single right-to-left line, and the surrounding container does
/// the positioning. Works the same for left-to-right text, so callers need no direction check.
#[must_use]
pub(crate) fn anchored_text<'a, Message: 'a>(
    content: Text<'a>,
    side: Horizontal,
) -> Element<'a, Message> {
    container(
        content
            .width(Length::Shrink)
            .align_x(TextAlignment::Default),
    )
    .width(Length::Fill)
    .align_x(side)
    .into()
}

/// Row whose children are given in inline order; reversed on screen for right-to-left.
#[must_use]
pub(crate) fn inline_row<'a, Message: 'a>(
    direction: Direction,
    mut children: Vec<Element<'a, Message>>,
) -> Row<'a, Message> {
    if direction.is_rtl() {
        children.reverse();
    }
    Row::with_children(children)
}

/// Screen x of a span `[x, x + width)` given in inline coordinates from the inline start of
/// `bounds_x..bounds_x + bounds_width`.
#[cfg(test)]
#[must_use]
pub(crate) fn mirror_x(
    direction: Direction,
    bounds_x: f32,
    bounds_width: f32,
    x: f32,
    width: f32,
) -> f32 {
    match direction {
        Direction::Ltr => bounds_x + x,
        Direction::Rtl => bounds_x + bounds_width - x - width,
    }
}
