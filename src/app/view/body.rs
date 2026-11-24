//! Header and body row: rail, splitter, results (toolbar and grid), splitter and detail pane in
//! inline order, mirrored by `inline_row` for right-to-left; drawers replace docked panes in the
//! narrow layout.

use iced::Element;

use crate::app::context::ViewContext;
use crate::app::message::Message;
use crate::app::state::App;

/// Header above the body row for the current mode.
#[must_use]
pub(super) fn body(app: &App, context: ViewContext) -> Element<'_, Message> {
    let _ = (app, context);
    iced::widget::space().into()
}

/// The result grid over `DataState::results`, or an empty element before data loads.
#[must_use]
pub(super) fn grid(app: &App, context: ViewContext) -> Element<'_, Message> {
    let _ = (app, context);
    iced::widget::space().into()
}
