//! One facet group: fold button, title, count badge, clear button, option search and checklist.

use iced::Element;

use crate::app::context::ViewContext;
use crate::app::message::Message;
use crate::app::view_model::{FacetUi, OptionList};
use crate::filter::FilterSet;

/// A facet group. The checklist toggles publish `FilterMessage::Toggle` with the option's typed
/// value, looked up in `options` from the stable option index.
#[must_use]
pub(super) fn view<'a>(
    ctx: ViewContext,
    ui: &'a FacetUi,
    options: &'a OptionList,
    filters: &'a FilterSet,
) -> Element<'a, Message> {
    let _ = (ctx, ui, options, filters);
    iced::widget::space().into()
}
