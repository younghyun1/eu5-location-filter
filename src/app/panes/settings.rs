//! Desktop Language and Appearance menus; the browser build never shows them.

use iced::Element;

use crate::app::context::ViewContext;
use crate::app::i18n::Locale;
use crate::app::message::Message;
use crate::app::state::Appearance;

/// Every locale by its native name; selecting one publishes `SettingsMessage::Locale`. Shows
/// `language.game_text_note` when the current locale has no game text.
#[must_use]
pub(crate) fn language_menu<'a>(ctx: ViewContext, current: Locale) -> Element<'a, Message> {
    let _ = (ctx, current);
    iced::widget::space().into()
}

/// Mode, accent swatches and animations choices.
#[must_use]
pub(crate) fn appearance_menu<'a>(
    ctx: ViewContext,
    appearance: &'a Appearance,
) -> Element<'a, Message> {
    let _ = (ctx, appearance);
    iced::widget::space().into()
}
