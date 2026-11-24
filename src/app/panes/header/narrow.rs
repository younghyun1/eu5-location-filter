//! Narrow header: Filters, Details and About on the first row, the search field on the second,
//! and a status line below.

use iced::widget::{Column, Id, container};
use iced::{Element, Length, Padding};

use super::frame::framed;
use super::search::search_slot;
use super::status::{self, HeaderStatus};
use crate::app::context::ViewContext;
use crate::app::i18n::Text;
use crate::app::ids;
use crate::app::layout::{
    BORDER_WIDTH, Drawer, NARROW_HEADER_GAP, NARROW_HEADER_HEIGHT, NARROW_HEADER_ROW_HEIGHT,
    NARROW_PADDING_INLINE, NARROW_STATUS_HEIGHT, inline_row,
};
use crate::app::message::{Message, OverlayMessage};
use crate::app::panes::controls::{badge, button_frame, button_label};
use crate::app::state::Surface;
use crate::app::theme::ButtonKind;

/// One of the three equal cells of the first row. `badge_count` adds the active-filter badge
/// after the label when it is not zero; `id` is the opener's focus identifier.
fn cell<'a>(
    ctx: ViewContext,
    label: &'a str,
    badge_count: usize,
    message: OverlayMessage,
    id: Id,
) -> Element<'a, Message> {
    let mut content = vec![button_label(ctx, label)];
    if badge_count > 0 {
        content.push(badge(ctx, badge_count));
    }
    let content = inline_row(ctx.direction, content).spacing(NARROW_HEADER_GAP);
    button_frame(
        ctx,
        content.into(),
        label,
        ButtonKind::Secondary,
        Length::Fill,
        Some(Message::Overlay(message)),
    )
    .id(id)
    .into()
}

/// Narrow header bar of height `NARROW_HEADER_HEIGHT`: two control rows of
/// `NARROW_HEADER_ROW_HEIGHT` and a status line, with the brand omitted because the host page
/// already names the application.
#[must_use]
pub(super) fn bar<'a>(
    ctx: ViewContext,
    search: &'a str,
    status: HeaderStatus,
) -> Element<'a, Message> {
    let filters = cell(
        ctx,
        ctx.t(Text::HeaderFilters),
        status.active_filters,
        OverlayMessage::OpenDrawer(Drawer::Filters),
        ids::FILTERS_BUTTON,
    );
    let details = cell(
        ctx,
        ctx.t(Text::HeaderDetails),
        0,
        OverlayMessage::OpenDrawer(Drawer::Details),
        ids::DETAILS_BUTTON,
    );
    let about = cell(
        ctx,
        ctx.t(Text::HeaderAbout),
        0,
        OverlayMessage::Open(Surface::About),
        ids::ABOUT_BUTTON,
    );
    let first = container(
        inline_row(ctx.direction, vec![filters, details, about])
            .spacing(NARROW_HEADER_GAP)
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .center_y(NARROW_HEADER_ROW_HEIGHT);
    let second = container(search_slot(ctx, search))
        .width(Length::Fill)
        .center_y(NARROW_HEADER_ROW_HEIGHT);
    let third = container(status::line(ctx, status))
        .width(Length::Fill)
        .center_y(NARROW_STATUS_HEIGHT - BORDER_WIDTH);
    let stack = Column::new().push(first).push(second).push(third);
    let padded = container(stack)
        .width(Length::Fill)
        .padding(Padding::from([0.0, NARROW_PADDING_INLINE]));
    framed(ctx, padded.into(), NARROW_HEADER_HEIGHT)
}
