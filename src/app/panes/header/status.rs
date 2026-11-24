//! Status values of the header and the text built from them.

use iced::alignment::Horizontal;
use iced::widget::text::{Alignment, Wrapping};
use iced::widget::{Column, text};
use iced::{Color, Element, Length};

use crate::app::context::ViewContext;
use crate::app::i18n::{Text, format};
use crate::app::layout::anchored_text;
use crate::app::layout::inline_row;
use crate::app::message::Message;
use crate::app::panes::controls::{counted, mono_font};
use crate::app::theme::TEXT_META;
use crate::model::EU5_GAME_VERSION;

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

/// Result count line; empty until a dataset is installed.
#[must_use]
pub(super) fn count_text(ctx: ViewContext, status: HeaderStatus) -> String {
    match status.result_count {
        Some(count) => counted(ctx, Text::HeaderResultCount, count),
        None => String::new(),
    }
}

/// Build line: the game version and data build, or the loading text before data arrives. The
/// build id is an identifier, so it is written without digit grouping.
#[must_use]
pub(super) fn build_text(ctx: ViewContext, status: HeaderStatus) -> String {
    match status.build_id {
        Some(build) => {
            let build = build.to_string();
            format(
                ctx.t(Text::HeaderBuild),
                &[("version", EU5_GAME_VERSION), ("build", build.as_str())],
            )
        }
        None => ctx.t(Text::HeaderBuildPending).to_owned(),
    }
}

/// One line of status text in Mono `TEXT_META` that never wraps.
fn meta_line<'a>(
    ctx: ViewContext,
    value: String,
    color: Color,
    side: Horizontal,
) -> Element<'a, Message> {
    anchored_text(
        text(value)
            .size(TEXT_META)
            .font(mono_font(ctx, false))
            .color(color)
            .wrapping(Wrapping::None),
        side,
    )
}

/// Two-line status block of the wide header: count in `ink`, build in `ink_faint`.
#[must_use]
pub(super) fn block<'a>(ctx: ViewContext, status: HeaderStatus) -> Element<'a, Message> {
    let palette = ctx.palette;
    let align = ctx.direction.start();
    Column::new()
        .push(meta_line(ctx, count_text(ctx, status), palette.ink, align))
        .push(meta_line(
            ctx,
            build_text(ctx, status),
            palette.ink_faint,
            align,
        ))
        .width(Length::Fill)
        .into()
}

/// One-line status of the narrow header: count at the inline start, build at the inline end.
#[must_use]
pub(super) fn line<'a>(ctx: ViewContext, status: HeaderStatus) -> Element<'a, Message> {
    let palette = ctx.palette;
    let count = meta_line(
        ctx,
        count_text(ctx, status),
        palette.ink,
        ctx.direction.start(),
    );
    let build = meta_line(
        ctx,
        build_text(ctx, status),
        palette.ink_faint,
        ctx.direction.end(),
    );
    inline_row(ctx.direction, vec![count, build])
        .width(Length::Fill)
        .into()
}
