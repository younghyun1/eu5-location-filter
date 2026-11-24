//! Header: brand, global search, result count and build text, and the header buttons; the
//! narrow layout stacks Filters, Details and About, the search, and a status line.

use iced::Element;

use crate::app::context::ViewContext;
use crate::app::message::Message;
use crate::app::state::Surface;

/// Status values the header shows.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct HeaderStatus {
    /// Result count, once data is loaded; formatted with `header.result_count`.
    pub(crate) result_count: Option<usize>,
    /// Steam build of the dataset; `None` shows `header.build.pending`.
    pub(crate) build_id: Option<u64>,
    /// Active filter fields, for the narrow Filters button badge.
    pub(crate) active_filters: usize,
}

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
    let _ = (ctx, search, status, brand_width, open);
    iced::widget::space().into()
}
