//! Entry point choosing the wide or narrow header.

use iced::Element;

use super::status::HeaderStatus;
use super::{narrow, wide};
use crate::app::context::ViewContext;
use crate::app::layout::LayoutMode;
use crate::app::message::Message;
use crate::app::state::Surface;

/// Header for the current layout mode. `search` is `FilterSet::search`; `brand_width` comes
/// from `layout::resolve`; `open` highlights the opener of an open menu.
#[must_use]
pub(crate) fn view<'a>(
    ctx: ViewContext,
    search: &'a str,
    status: HeaderStatus,
    brand_width: f32,
    open: Option<Surface>,
) -> Element<'a, Message> {
    match ctx.mode {
        LayoutMode::Wide => wide::bar(ctx, search, status, brand_width, open),
        LayoutMode::Narrow => narrow::bar(ctx, search, status),
    }
}
