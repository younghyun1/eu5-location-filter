//! Rail frame and section order.

use iced::Element;

use crate::app::context::ViewContext;
use crate::app::message::Message;
use crate::app::state::TextFields;
use crate::app::view_model::{FacetId, FacetOptions, FacetUi};
use crate::filter::FilterSet;

/// Everything the rail shows.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RailModel<'a> {
    /// Active filters.
    pub(crate) filters: &'a FilterSet,
    /// Option lists; `None` before data loads, when groups show no options.
    pub(crate) options: Option<&'a FacetOptions>,
    /// Per-group state in `FacetId::ALL` order.
    pub(crate) facets: &'a [FacetUi; FacetId::COUNT],
    /// Field buffers and inline errors.
    pub(crate) fields: &'a TextFields,
}

/// The rail filling the width its parent gives it: the collapsed strip when `collapsed`, and a
/// Close button instead of the collapse button when `in_drawer`.
#[must_use]
pub(crate) fn view<'a>(
    ctx: ViewContext,
    rail: RailModel<'a>,
    collapsed: bool,
    in_drawer: bool,
) -> Element<'a, Message> {
    let _ = (ctx, rail, collapsed, in_drawer);
    iced::widget::space().into()
}
