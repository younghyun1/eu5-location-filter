//! Load-error card over the error veil.

use iced::font::Weight;
use iced::widget::text::{Alignment, Wrapping};
use iced::widget::{Column, container, scrollable};
use iced::{Element, Length, Theme};

use super::kit::{card_frame, mono, prose, sans};
use crate::AppError;
use crate::app::context::ViewContext;
use crate::app::i18n::{Text, format};
use crate::app::ids;
use crate::app::layout::{
    BUTTON_WIDTH, DIALOG_GAP, ERROR_CARD_WIDTH, MENU_MAX_HEIGHT, MENU_PADDING, inline_row,
};
use crate::app::message::{LoadMessage, Message};
use crate::app::panes::controls::{text_button, text_field};
use crate::app::state::LoadFailure;
use crate::app::theme::{
    self, ButtonKind, TEXT_BODY, TEXT_CAPTION, TEXT_HEADING, TEXT_META, TEXT_SMALL,
};

/// Message of the error's family, with the placeholders its catalog text names. `Fetch` and
/// `PackMismatch` have no family text of their own yet: a failed request reads as a failed
/// file operation naming its message, and a mismatched pack as invalid data.
#[must_use]
pub(super) fn family_message(ctx: ViewContext, error: &AppError) -> String {
    match error {
        AppError::Io { path, .. } => format(
            ctx.t(Text::ErrorIo),
            &[("path", path.display().to_string().as_str())],
        ),
        AppError::Parse {
            source_name,
            offset,
            ..
        } => format(
            ctx.t(Text::ErrorParse),
            &[
                ("source", source_name.as_str()),
                ("offset", offset.to_string().as_str()),
            ],
        ),
        AppError::Installation(_) => ctx.t(Text::ErrorInstallation).to_owned(),
        AppError::MissingFile(path) => format(
            ctx.t(Text::ErrorMissingFile),
            &[("path", path.display().to_string().as_str())],
        ),
        AppError::InvalidData(_) | AppError::PackMismatch(_) => {
            ctx.t(Text::ErrorInvalidData).to_owned()
        }
        AppError::Encoding(_) => ctx.t(Text::ErrorEncoding).to_owned(),
        AppError::Compression(_) => ctx.t(Text::ErrorCompression).to_owned(),
        AppError::Fetch(message) => format(ctx.t(Text::ErrorIo), &[("path", message.as_str())]),
        #[cfg(feature = "desktop")]
        AppError::Png { path, .. } => format(
            ctx.t(Text::ErrorPng),
            &[("path", path.display().to_string().as_str())],
        ),
        AppError::Ui(_) => ctx.t(Text::ErrorUi).to_owned(),
        AppError::Worker => ctx.t(Text::ErrorWorker).to_owned(),
    }
}

/// Hint under the detail: the browser can only reload; the desktop distinguishes errors that a
/// retry can fix from those that need a rebuild.
#[must_use]
pub(super) fn hint_text(ctx: ViewContext, error: &AppError) -> Text {
    if !ctx.platform.has_desktop_controls() {
        Text::ErrorHintWeb
    } else if error.is_retryable() {
        Text::ErrorHintRetryable
    } else {
        Text::ErrorHintRebuild
    }
}

/// Title, family message for the error variant, technical detail, hint, and on desktop the
/// installation directory field with Retry and Rebuild data.
#[must_use]
pub(crate) fn error_card<'a>(
    ctx: ViewContext,
    failure: &'a LoadFailure,
    install_path: &'a str,
) -> Element<'a, Message> {
    let palette = ctx.palette;
    let error: &AppError = &failure.error;
    let title = sans(
        ctx,
        ctx.t(Text::ErrorTitle),
        TEXT_HEADING,
        Weight::Semibold,
        palette.danger,
    )
    .width(Length::Fill);
    let message = prose(ctx, family_message(ctx, error), TEXT_BODY, palette.ink);
    // Technical text stays left to right in every layout direction.
    let detail = mono(ctx, error.to_string(), TEXT_META, false, palette.ink_muted)
        .align_x(Alignment::Left)
        .width(Length::Fill)
        .wrapping(Wrapping::WordOrGlyph);
    let detail_block = container(
        scrollable(detail)
            .width(Length::Fill)
            .style(move |_: &Theme, status| theme::scrollable(palette, status)),
    )
    .padding(MENU_PADDING)
    .width(Length::Fill)
    .max_height(MENU_MAX_HEIGHT / 3.0)
    .style(move |_: &Theme| theme::detail_block(palette));
    let mut column = Column::new()
        .push(title)
        .push(message)
        .push(mono(
            ctx,
            ctx.t(Text::ErrorDetail),
            TEXT_CAPTION,
            true,
            palette.ink_faint,
        ))
        .push(detail_block)
        .push(prose(
            ctx,
            ctx.t(hint_text(ctx, error)),
            TEXT_SMALL,
            palette.ink_muted,
        ))
        .spacing(DIALOG_GAP)
        .width(Length::Fill);
    if ctx.platform.has_desktop_controls() {
        column = column.push(desktop_controls(ctx, install_path));
    }
    card_frame(ctx, column.into(), ERROR_CARD_WIDTH)
}

/// Installation directory field with Retry (primary) and Rebuild data.
fn desktop_controls<'a>(ctx: ViewContext, install_path: &'a str) -> Element<'a, Message> {
    let label = prose(
        ctx,
        ctx.t(Text::ErrorInstallPathLabel),
        TEXT_SMALL,
        ctx.palette.ink_muted,
    );
    let field = text_field(
        ctx,
        ids::INSTALL_PATH,
        ctx.t(Text::ErrorInstallPathPlaceholder),
        install_path,
        false,
        |value| Message::Load(LoadMessage::InstallPath(value)),
    );
    let retry = text_button(
        ctx,
        ctx.t(Text::ErrorRetry),
        ButtonKind::Primary,
        BUTTON_WIDTH,
        Some(Message::Load(LoadMessage::Retry)),
    );
    let rebuild = text_button(
        ctx,
        ctx.t(Text::ErrorRebuild),
        ButtonKind::Secondary,
        BUTTON_WIDTH,
        Some(Message::Load(LoadMessage::Rebuild)),
    );
    let buttons = inline_row(ctx.direction, vec![retry.into(), rebuild.into()])
        .spacing(MENU_PADDING)
        .width(Length::Fill);
    Column::new()
        .push(label)
        .push(field)
        .push(buttons)
        .spacing(MENU_PADDING)
        .width(Length::Fill)
        .into()
}

#[cfg(test)]
mod tests {
    use super::{error_card, family_message, hint_text};
    use crate::AppError;
    use crate::app::context::Platform;
    use crate::app::i18n::Text;
    use crate::app::layout::LayoutMode;
    use crate::app::panes::overlays::kit::test_support::{context, contexts};
    use crate::app::state::LoadFailure;
    use crate::app::i18n::Locale;
    use std::sync::Arc;

    fn english(platform: Platform) -> crate::app::context::ViewContext {
        context(platform, LayoutMode::Wide, Locale::EnUs)
    }

    fn io_error() -> AppError {
        AppError::io(
            "read",
            "lang/ko.bitcode.zst",
            std::io::Error::from(std::io::ErrorKind::NotFound),
        )
    }

    #[test]
    fn family_messages_fill_their_placeholders() {
        let ctx = english(Platform::Desktop);
        let parse = AppError::parse("hierarchy.txt", 412, "unexpected token");
        let shown = family_message(ctx, &parse);
        assert!(shown.contains("hierarchy.txt") && shown.contains("412"), "{shown}");
        let shown = family_message(ctx, &io_error());
        assert!(shown.contains("lang/ko.bitcode.zst"), "{shown}");
        let shown = family_message(ctx, &AppError::MissingFile("common/a.txt".into()));
        assert!(shown.contains("common/a.txt"), "{shown}");
        let shown = family_message(ctx, &AppError::Fetch("status 503".to_owned()));
        assert!(shown.contains("status 503"), "{shown}");
    }

    #[test]
    fn family_messages_have_no_unresolved_placeholder() {
        let ctx = english(Platform::Desktop);
        let errors = [
            io_error(),
            AppError::parse("x", 1, "y"),
            AppError::Installation("none".to_owned()),
            AppError::MissingFile("a".into()),
            AppError::InvalidData("a".to_owned()),
            AppError::PackMismatch("a".to_owned()),
            AppError::Encoding("a".to_owned()),
            AppError::Compression("a".to_owned()),
            AppError::Fetch("a".to_owned()),
            AppError::Ui("a".to_owned()),
            AppError::Worker,
        ];
        for error in &errors {
            let shown = family_message(ctx, error);
            assert!(!shown.is_empty(), "{error:?}");
            assert!(!shown.contains('{') && !shown.contains('}'), "{shown}");
        }
    }

    #[test]
    fn plain_families_use_their_catalog_text() {
        let ctx = english(Platform::Desktop);
        assert_eq!(
            family_message(ctx, &AppError::Worker),
            ctx.t(Text::ErrorWorker)
        );
        assert_eq!(
            family_message(ctx, &AppError::PackMismatch("a".to_owned())),
            ctx.t(Text::ErrorInvalidData)
        );
    }

    #[test]
    fn the_hint_follows_platform_and_retryability() {
        let desktop = english(Platform::Desktop);
        let browser = english(Platform::Browser);
        assert_eq!(hint_text(desktop, &io_error()), Text::ErrorHintRetryable);
        assert_eq!(
            hint_text(desktop, &AppError::InvalidData("a".to_owned())),
            Text::ErrorHintRebuild
        );
        assert_eq!(hint_text(browser, &io_error()), Text::ErrorHintWeb);
        assert_eq!(
            hint_text(browser, &AppError::InvalidData("a".to_owned())),
            Text::ErrorHintWeb
        );
    }

    #[test]
    fn the_card_builds_for_every_context() {
        let failure = LoadFailure {
            error: Arc::new(io_error()),
        };
        for ctx in contexts() {
            let _card = error_card(ctx, &failure, "/games/eu5");
        }
    }
}
