//! Drawer frames, the scrim behind drawers and dialogs, and the centred modal frame.

use iced::alignment::Horizontal;
use iced::widget::{container, mouse_area, opaque, pin, rule, space};
use iced::{Element, Length, Theme};

use super::kit::open_surface;
use crate::app::context::ViewContext;
use crate::app::layout::{BORDER_WIDTH, DIALOG_MARGIN, Direction, Drawer};
use crate::app::message::Message;
use crate::app::theme;

/// Where a drawer sits and how much of it shows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Slide {
    /// Screen edge the drawer is attached to.
    pub(super) side: Horizontal,
    /// Width of the visible strip at the current progress.
    pub(super) visible: f32,
    /// Horizontal offset of the full-width panel inside the visible strip; never positive.
    pub(super) offset: f32,
}

/// Screen edge of a drawer: the filters drawer attaches to the inline start and the details
/// drawer to the inline end, so both swap sides in right-to-left layouts.
#[must_use]
pub(super) const fn drawer_side(direction: Direction, drawer: Drawer) -> Horizontal {
    match drawer {
        Drawer::Filters => direction.start(),
        Drawer::Details => direction.end(),
    }
}

/// Geometry of a drawer `progress` of the way open. The panel keeps its full width and is
/// moved behind a clipping strip that grows from the screen edge, so nothing is laid out at a
/// width other than `width` and the panel's widget state survives the animation. A non-finite
/// or out-of-range `progress` is clamped to `0.0..=1.0`.
#[must_use]
pub(super) fn slide(direction: Direction, drawer: Drawer, width: f32, progress: f32) -> Slide {
    let progress = if progress.is_finite() {
        progress.clamp(0.0, 1.0)
    } else {
        1.0
    };
    let width = width.max(0.0);
    Slide {
        side: drawer_side(direction, drawer),
        visible: width * progress,
        offset: -(width * (1.0 - progress)),
    }
}

/// A drawer of `width` sliding in from its inline edge (filters from the start, details from
/// the end); `progress` in `0..=1` is the slide position from the drawer animation.
///
/// The edge facing the window carries a hairline instead of a shadow, because the strip that
/// clips the slide would cut a shadow off. The attached edge slides; the other edge reveals the
/// panel in place, since the panel cannot be laid out wider than the strip that holds it.
#[must_use]
pub(crate) fn drawer<'a>(
    ctx: ViewContext,
    drawer: Drawer,
    width: f32,
    progress: f32,
    content: Element<'a, Message>,
) -> Element<'a, Message> {
    let palette = ctx.palette;
    let geometry = slide(ctx.direction, drawer, width, progress);
    let panel = container(content)
        .width(Length::Fixed((width - BORDER_WIDTH).max(0.0)))
        .height(Length::Fill)
        .style(move |_: &Theme| theme::surface(palette));
    let edge = rule::vertical(BORDER_WIDTH).style(move |_: &Theme| theme::rule(palette, false));
    // The hairline faces the window: after the panel on the left edge, before it on the right.
    let children: Vec<Element<'a, Message>> = if geometry.side == Horizontal::Left {
        vec![panel.into(), edge.into()]
    } else {
        vec![edge.into(), panel.into()]
    };
    let framed = iced::widget::Row::with_children(children)
        .width(Length::Fixed(width.max(0.0)))
        .height(Length::Fill);
    let strip = container(pin(framed).x(geometry.offset))
        .width(Length::Fixed(geometry.visible))
        .height(Length::Fill)
        .clip(true);
    // Not opaque: the caller already wraps the content, and the scrim beneath must still
    // receive presses outside the strip.
    open_surface(
        container(strip)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(geometry.side)
            .into(),
    )
}

/// Full-window scrim at `opacity` that publishes `on_press` when pressed and blocks input to
/// everything beneath.
#[must_use]
pub(crate) fn scrim<'a>(ctx: ViewContext, opacity: f32, on_press: Message) -> Element<'a, Message> {
    let palette = ctx.palette;
    let ground = container(space())
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_: &Theme| theme::scrim(palette, opacity));
    opaque(mouse_area(ground).on_press(on_press))
}

/// Centres a dialog card within the viewport, at least `DIALOG_MARGIN` from its edges.
#[must_use]
pub(crate) fn modal<'a>(_ctx: ViewContext, card: Element<'a, Message>) -> Element<'a, Message> {
    open_surface(
        container(card)
            .padding(DIALOG_MARGIN)
            .center(Length::Fill)
            .into(),
    )
}

#[cfg(test)]
mod tests {
    use iced::alignment::Horizontal;

    use super::{Slide, drawer, drawer_side, modal, scrim, slide};
    use crate::app::layout::{Direction, Drawer};
    use crate::app::message::{Message, OverlayMessage};
    use crate::app::panes::overlays::kit::test_support::contexts;

    #[test]
    fn drawers_attach_to_their_inline_edge() {
        assert_eq!(drawer_side(Direction::Ltr, Drawer::Filters), Horizontal::Left);
        assert_eq!(drawer_side(Direction::Ltr, Drawer::Details), Horizontal::Right);
        assert_eq!(drawer_side(Direction::Rtl, Drawer::Filters), Horizontal::Right);
        assert_eq!(drawer_side(Direction::Rtl, Drawer::Details), Horizontal::Left);
    }

    #[test]
    fn a_closed_drawer_shows_nothing_and_an_open_one_everything() {
        let closed = slide(Direction::Ltr, Drawer::Filters, 320.0, 0.0);
        assert_eq!(
            closed,
            Slide {
                side: Horizontal::Left,
                visible: 0.0,
                offset: -320.0
            }
        );
        let open = slide(Direction::Ltr, Drawer::Filters, 320.0, 1.0);
        assert!((open.visible - 320.0).abs() < f32::EPSILON);
        assert!(open.offset.abs() < f32::EPSILON);
    }

    #[test]
    fn the_visible_strip_and_the_hidden_part_always_add_up_to_the_width() {
        for step in 0..=10_u8 {
            let progress = f32::from(step) / 10.0;
            let geometry = slide(Direction::Rtl, Drawer::Details, 360.0, progress);
            assert!((geometry.visible + geometry.offset.abs() - 360.0).abs() < 0.01);
            assert!(geometry.offset <= 0.0);
        }
    }

    #[test]
    fn stray_progress_is_clamped() {
        let over = slide(Direction::Ltr, Drawer::Filters, 100.0, 4.0);
        assert!((over.visible - 100.0).abs() < f32::EPSILON);
        let under = slide(Direction::Ltr, Drawer::Filters, 100.0, -2.0);
        assert!(under.visible.abs() < f32::EPSILON);
        let not_a_number = slide(Direction::Ltr, Drawer::Filters, 100.0, f32::NAN);
        assert!((not_a_number.visible - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn the_frames_build_for_every_context() {
        for ctx in contexts() {
            for progress in [0.0, 0.5, 1.0] {
                let _drawer = drawer(
                    ctx,
                    Drawer::Details,
                    360.0,
                    progress,
                    iced::widget::space().into(),
                );
            }
            let _scrim = scrim(ctx, 0.5, Message::Overlay(OverlayMessage::Close));
            let _modal = modal(ctx, iced::widget::space().into());
        }
    }
}
