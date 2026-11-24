//! Loading card over the loading veil.

use std::borrow::Cow;

use iced::font::Weight;
use iced::widget::{Column, progress_bar};
use iced::{Element, Length, Theme};

use super::kit::{card_frame, mono, sans};
use crate::app::context::ViewContext;
use crate::app::i18n::{Text, format};
use crate::app::layout::{DIALOG_GAP, LOADING_CARD_WIDTH, PROGRESS_HEIGHT};
use crate::app::message::Message;
use crate::app::state::LoadProgress;
use crate::app::theme::{self, TEXT_META, TEXT_TITLE};
use crate::model::EU5_GAME_VERSION;

/// Stage text of the loading card. Two stages name the game version, which comes from
/// `EU5_GAME_VERSION` and not from the catalog; every other stage is plain catalog text.
#[must_use]
pub(super) fn stage_text(ctx: ViewContext, stage: Text) -> Cow<'static, str> {
    let template = ctx.t(stage);
    match stage {
        Text::LoadLoadingData | Text::LoadPreparing => {
            Cow::Owned(format(template, &[("version", EU5_GAME_VERSION)]))
        }
        _ => Cow::Borrowed(template),
    }
}

/// Whole percent of a reported fraction; a value outside `0.0..=1.0` or not finite is clamped,
/// so a stray report can never draw a bar or a number outside its range.
#[must_use]
pub(super) fn percent(fraction: f32) -> u64 {
    if !fraction.is_finite() {
        return 0;
    }
    let whole = (fraction.clamp(0.0, 1.0) * 100.0).round();
    // Rounded value is within 0..=100, so the conversion cannot truncate or wrap.
    whole as u64
}

/// Stage text, progress bar and `format.progress`; the bar reflects reported progress only.
#[must_use]
pub(crate) fn loading_card<'a>(ctx: ViewContext, progress: LoadProgress) -> Element<'a, Message> {
    let palette = ctx.palette;
    let stage = sans(
        ctx,
        stage_text(ctx, progress.stage),
        TEXT_TITLE,
        Weight::Semibold,
        palette.ink,
    )
    .width(Length::Fill);
    let fraction = if progress.fraction.is_finite() {
        progress.fraction.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let bar = progress_bar(0.0..=1.0, fraction)
        .girth(PROGRESS_HEIGHT)
        .style(move |_: &Theme| theme::progress_bar(palette));
    let digits = percent(progress.fraction).to_string();
    let label = mono(
        ctx,
        format(ctx.t(Text::FormatProgress), &[("percent", digits.as_str())]),
        TEXT_META,
        false,
        palette.ink_muted,
    )
    .width(Length::Fill);
    let column = Column::new()
        .push(stage)
        .push(bar)
        .push(label)
        .spacing(DIALOG_GAP)
        .width(Length::Fill);
    card_frame(ctx, column.into(), LOADING_CARD_WIDTH)
}

#[cfg(test)]
mod tests {
    use super::{loading_card, percent, stage_text};
    use crate::app::i18n::Text;
    use crate::app::panes::overlays::kit::test_support::contexts;
    use crate::app::state::LoadProgress;
    use crate::model::EU5_GAME_VERSION;

    #[test]
    fn percent_rounds_and_clamps() {
        assert_eq!(percent(0.0), 0);
        assert_eq!(percent(0.004), 0);
        assert_eq!(percent(0.006), 1);
        assert_eq!(percent(0.5), 50);
        assert_eq!(percent(1.0), 100);
        assert_eq!(percent(7.0), 100);
        assert_eq!(percent(-3.0), 0);
        assert_eq!(percent(f32::NAN), 0);
        assert_eq!(percent(f32::INFINITY), 0);
    }

    #[test]
    fn version_stages_name_the_game_version() {
        for ctx in contexts() {
            for stage in [Text::LoadLoadingData, Text::LoadPreparing] {
                let shown = stage_text(ctx, stage);
                assert!(shown.contains(EU5_GAME_VERSION), "{shown}");
                assert!(!shown.contains("{version}"), "{shown}");
            }
        }
    }

    #[test]
    fn plain_stages_are_borrowed_catalog_text() {
        let ctx = contexts().into_iter().next();
        let Some(ctx) = ctx else {
            return;
        };
        for stage in [Text::LoadStarting, Text::LoadEmbedded, Text::ImportRivers] {
            assert_eq!(stage_text(ctx, stage), ctx.t(stage));
        }
    }

    #[test]
    fn the_card_builds_for_every_context() {
        for ctx in contexts() {
            let _card = loading_card(
                ctx,
                LoadProgress {
                    stage: Text::LoadLoadingData,
                    fraction: 0.42,
                },
            );
        }
    }
}
