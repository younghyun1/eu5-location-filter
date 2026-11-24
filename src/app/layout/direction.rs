//! Layout direction and the helpers that mirror geometry for right-to-left locales. Direction
//! changes geometry only; colours and metrics stay the same.

use iced::Element;
use iced::advanced::text::Alignment as TextAlignment;
use iced::alignment::Horizontal;
use iced::widget::Row;

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

    /// Text alignment of the inline start for stock text widgets with a finite width. Custom
    /// widgets drawing single lines with infinite bounds use `TextAlignment::Default` instead,
    /// because iced relayouts right-to-left lines only for that alignment.
    #[must_use]
    pub(crate) const fn text_start(self) -> TextAlignment {
        match self {
            Self::Ltr => TextAlignment::Left,
            Self::Rtl => TextAlignment::Right,
        }
    }

    /// Text alignment of the inline end, for numbers in stock widgets.
    #[must_use]
    pub(crate) const fn text_end(self) -> TextAlignment {
        match self {
            Self::Ltr => TextAlignment::Right,
            Self::Rtl => TextAlignment::Left,
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
