//! Key navigation, the tap test and right-to-left label detection used by the checklist.

use iced::Point;

use crate::app::layout::TOUCH_SLOP;

/// A keyboard move of the active option.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::app::widget::checklist) enum Nav {
    /// ArrowUp.
    Previous,
    /// ArrowDown.
    Next,
    /// PageUp.
    PageUp,
    /// PageDown.
    PageDown,
    /// Home.
    First,
    /// End.
    Last,
}

/// New active row after a move from `current`, clamped to the rows; `None` without rows.
/// The caller passes the row it draws as active, so a move after a Tab focus starts from the
/// outlined row.
#[must_use]
pub(in crate::app::widget::checklist) fn navigate(
    current: usize,
    nav: Nav,
    count: usize,
    page: usize,
) -> Option<usize> {
    let last = count.checked_sub(1)?;
    let target = match nav {
        Nav::First => 0,
        Nav::Last => last,
        Nav::Previous => current.saturating_sub(1),
        Nav::Next => current.saturating_add(1),
        Nav::PageUp => current.saturating_sub(page),
        Nav::PageDown => current.saturating_add(page),
    };
    Some(target.min(last))
}

/// Whether a touch that moved from `start` to `now` is still a tap.
#[must_use]
pub(in crate::app::widget::checklist) fn is_tap(start: Point, now: Point) -> bool {
    let dx = now.x - start.x;
    let dy = now.y - start.y;
    dx.hypot(dy) <= TOUCH_SLOP
}

/// Whether a label holds right-to-left script (Hebrew, Arabic, Syriac, Thaana, N'Ko and their
/// presentation forms). Such labels need a measured width to end at the checkbox, because
/// iced positions right-aligned single-line right-to-left text incorrectly against an infinite
/// width.
#[must_use]
pub(in crate::app::widget::checklist) fn has_rtl_script(label: &str) -> bool {
    label.chars().any(|c| matches!(c, '\u{0590}'..='\u{08FF}' | '\u{FB1D}'..='\u{FDFF}' | '\u{FE70}'..='\u{FEFF}'))
}
