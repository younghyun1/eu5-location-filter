//! Application callbacks passed to the iced builder.

use iced::Element;

use crate::app::i18n::{Text, t};
use crate::app::message::Message;
use crate::app::state::App;

/// Whole interface: header, notice bar, body row, then transient surfaces stacked in the
/// contract's draw order (column menu, notice, drawers with scrim, About, loading card, error
/// card), with only the topmost modal surface receiving pointer input.
#[must_use]
pub(crate) fn view(app: &App) -> Element<'_, Message> {
    let context = app.view_context();
    let body = super::body::body(app, context);
    super::surfaces::with_surfaces(app, context, body)
}

/// Window title, `app.title` in the interface locale.
#[must_use]
pub(crate) fn title(app: &App) -> String {
    t(app.locale, Text::AppTitle).to_owned()
}

/// iced theme matching the current palette, for stock widget defaults.
#[must_use]
pub(crate) fn theme(app: &App) -> iced::Theme {
    app.appearance.theme.clone()
}

/// Window background and default text colour from the current palette.
#[must_use]
pub(crate) fn window_style(app: &App, _theme: &iced::Theme) -> iced::theme::Style {
    iced::theme::Style {
        background_color: app.appearance.palette.paper,
        text_color: app.appearance.palette.ink,
    }
}
