//! One facet group: fold button, title, count badge, clear button, option search and checklist.

use iced::alignment::Vertical;
use iced::font::Weight;
use iced::widget::text::Wrapping;
use iced::widget::{Column, container, text};
use iced::{Element, Length};

use crate::app::context::ViewContext;
use crate::app::i18n::Text;
use crate::app::ids;
use crate::app::layout::{
    CHECK_LABEL_GAP, CHECKLIST_MAX_VISIBLE_ROWS, CHECKLIST_ROW_HEIGHT, CHECKLIST_ROW_HEIGHT_TOUCH,
    CHECKLIST_SEARCH_THRESHOLD, RANGE_GAP, inline_row,
};
use crate::app::message::{FilterMessage, Message};
use crate::app::panes::controls::{
    badge, button_frame, counted, icon_button, sans_font, text_field, with_tooltip_owned,
};
use crate::app::theme::{ButtonKind, Icon, TEXT_SMALL};
use crate::app::view_model::{FacetRows, FacetUi, OptionList, selected_count};
use crate::app::widget::Checklist;
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
    let count = selected_count(filters, ui.facet);
    let mut group = Column::new()
        .spacing(RANGE_GAP / 2.0)
        .width(Length::Fill)
        .push(header(ctx, ui, count));
    if !ui.folded {
        if options.len() > CHECKLIST_SEARCH_THRESHOLD {
            group = group.push(search(ctx, ui));
        }
        group = group.push(checklist(ctx, ui, options, filters));
    }
    group.into()
}

/// Header row: the fold button with chevron and title, the selection badge, and Clear. The
/// badge stays while the group is folded; Clear appears only with a selection.
fn header<'a>(ctx: ViewContext, ui: &'a FacetUi, count: usize) -> Element<'a, Message> {
    let facet = ui.facet;
    let palette = ctx.palette;
    let chevron = if ui.folded {
        Icon::Folded
    } else {
        Icon::SortDescending
    };
    let glyph = text(chevron.glyph(ctx.direction).to_string())
        .size(TEXT_SMALL)
        .font(chevron.font())
        .color(palette.ink_muted);
    let title = text(ctx.t(facet.title()))
        .size(TEXT_SMALL)
        .font(sans_font(ctx, Weight::Medium))
        .color(palette.ink)
        .wrapping(Wrapping::None);
    let label = container(
        inline_row(ctx.direction, vec![glyph.into(), title.into()])
            .spacing(CHECK_LABEL_GAP)
            .align_y(Vertical::Center),
    )
    .width(Length::Fill)
    .align_x(ctx.direction.start())
    .center_y(Length::Fill);
    let tip = ctx.t(if ui.folded {
        Text::ChecklistExpand
    } else {
        Text::ChecklistCollapse
    });
    let fold = button_frame(
        ctx,
        label.into(),
        tip,
        ButtonKind::Ghost,
        Length::Fill,
        Some(Message::Filter(FilterMessage::Fold {
            facet,
            folded: !ui.folded,
        })),
    )
    .id(ids::facet_fold(facet));
    let mut items: Vec<Element<'a, Message>> = vec![fold.into()];
    if count > 0 {
        items.push(with_tooltip_owned(
            ctx,
            badge(ctx, count),
            counted(ctx, Text::ChecklistSelectedCount, count),
        ));
        items.push(
            icon_button(
                ctx,
                Icon::Close,
                ctx.t(Text::ChecklistClear),
                Some(Message::Filter(FilterMessage::ClearFacet(facet))),
            )
            .into(),
        );
    }
    inline_row(ctx.direction, items)
        .spacing(RANGE_GAP)
        .align_y(Vertical::Center)
        .width(Length::Fill)
        .into()
}

/// Option search field of the group.
fn search<'a>(ctx: ViewContext, ui: &'a FacetUi) -> Element<'a, Message> {
    let facet = ui.facet;
    text_field(
        ctx,
        ids::facet_search(facet),
        ctx.t(Text::ChecklistSearchPlaceholder),
        &ui.query,
        false,
        move |query| Message::Filter(FilterMessage::OptionQuery { facet, query }),
    )
}

/// The checklist of visible options.
fn checklist<'a>(
    ctx: ViewContext,
    ui: &'a FacetUi,
    options: &'a OptionList,
    filters: &'a FilterSet,
) -> Element<'a, Message> {
    let facet = ui.facet;
    let folded = ui.folded;
    let narrow = ctx.is_narrow();
    let rows = FacetRows {
        facet,
        options,
        visible: &ui.visible,
        filters,
    };
    Checklist::new(rows)
        .id(ids::facet_list(facet))
        .direction(ctx.direction)
        .palette(ctx.palette)
        .font(sans_font(ctx, Weight::Normal))
        .row_height(if narrow {
            CHECKLIST_ROW_HEIGHT_TOUCH
        } else {
            CHECKLIST_ROW_HEIGHT
        })
        .max_visible_rows(CHECKLIST_MAX_VISIBLE_ROWS)
        .touch(narrow)
        .empty_text(ctx.t(Text::ChecklistEmpty))
        // An index outside the list cannot come from the widget; answering with the group's
        // current fold state keeps the callback total without changing any state.
        .on_toggle(move |index, selected| match options.value(index) {
            Some(value) => Message::Filter(FilterMessage::Toggle {
                facet,
                value,
                selected,
            }),
            None => Message::Filter(FilterMessage::Fold { facet, folded }),
        })
        .into()
}
