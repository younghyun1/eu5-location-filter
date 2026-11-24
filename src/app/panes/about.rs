//! About dialog: title, version and data build, author, links, disclaimer, licences with the
//! expandable full texts, the keyboard table, and Close.

use iced::Element;

use crate::app::context::ViewContext;
use crate::app::message::Message;

/// Dialog card content; the caller centres it over a scrim. `build_id` is `None` before data
/// loads.
#[must_use]
pub(crate) fn view<'a>(
    ctx: ViewContext,
    build_id: Option<u64>,
    licenses_expanded: bool,
) -> Element<'a, Message> {
    let _ = (ctx, build_id, licenses_expanded);
    iced::widget::space().into()
}
