//! Detail pane: pinned header with the hide button, then the colour block, name, key,
//! breadcrumb and field groups of the selected location, or the empty state.

use iced::Element;

use crate::app::context::ViewContext;
use crate::app::message::Message;
use crate::app::view_model::LocationDetail;

/// The pane filling the width its parent gives it; a Close button replaces the hide button
/// when `in_drawer`.
#[must_use]
pub(crate) fn view<'a>(
    ctx: ViewContext,
    detail: Option<&'a LocationDetail>,
    in_drawer: bool,
) -> Element<'a, Message> {
    let _ = (ctx, detail, in_drawer);
    iced::widget::space().into()
}
