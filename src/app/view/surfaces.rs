//! Transient surfaces stacked over the body in the contract's draw order, from back to front:
//! popover menu (above a transparent layer that closes it on an outside press), notice bar,
//! drawer with its scrim, About dialog over a scrim, then the loading or error veil.
//!
//! Only the topmost modal surface receives pointer input: menus, the notice, drawers and the
//! About card are wrapped in `opaque` so presses on them never reach the layers beneath, the
//! scrims close their surface on a press outside the card or drawer, and the veils block the
//! whole window.

use iced::alignment::Vertical;
use iced::widget::{Stack, container, mouse_area, opaque, space};
use iced::{Element, Length, Padding};

use super::body::rail_model;
use super::plan::{self, Menu, MenuAnchor};
use crate::app::context::ViewContext;
use crate::app::i18n::Text;
use crate::app::layout::{Direction, Drawer};
use crate::app::message::{Message, OverlayMessage};
use crate::app::panes::{about, column_menu, detail, filter_rail, overlays, settings};
use crate::app::state::{App, LoadProgress, Phase};
use crate::app::theme::{self, Veil};

/// Stacks the open menu or dialog, notice, drawer, loading card and error card over `body`.
#[must_use]
pub(super) fn with_surfaces<'a>(
    app: &'a App,
    context: ViewContext,
    body: Element<'a, Message>,
) -> Element<'a, Message> {
    let panes = crate::app::layout::resolve(app.viewport.width, &app.panes, app.layout_mode);
    let surfaces = plan::plan(
        app.overlay.surface,
        app.overlay.notice,
        &panes,
        app.layout_mode,
    );
    let mut layers: Vec<Element<'a, Message>> = Vec::with_capacity(9);
    layers.push(body);
    if let Some(menu) = surfaces.menu {
        layers.push(outside_press());
        let anchor = plan::menu_anchor(menu, app.layout_mode, &panes, app.viewport);
        layers.push(popover(context, anchor, menu_content(app, context, menu)));
    }
    if let Some(notice) = surfaces.notice {
        layers.push(
            container(opaque(overlays::notice_bar(context, notice)))
                .padding(Padding::ZERO.top(plan::header_height(app.layout_mode)))
                .width(Length::Fill)
                .into(),
        );
    }
    if let Some((drawer, width)) = surfaces.drawer {
        // Sampled at the time of the latest frame message so view stays a pure function of
        // the state; frames keep arriving while the animation runs.
        let progress = app
            .overlay
            .drawer_motion
            .interpolate(0.0_f32, 1.0_f32, app.appearance.now);
        layers.push(overlays::scrim(
            context,
            progress,
            Message::Overlay(OverlayMessage::CloseDrawer),
        ));
        layers.push(overlays::drawer(
            context,
            drawer,
            width,
            progress,
            opaque(drawer_content(app, context, drawer)),
        ));
    }
    if surfaces.about {
        let build_id = app.data.as_ref().map(|data| data.dataset.stored.build_id);
        layers.push(overlays::scrim(
            context,
            1.0,
            Message::Overlay(OverlayMessage::Close),
        ));
        layers.push(overlays::modal(
            context,
            opaque(about::view(context, build_id, app.overlay.licenses_expanded)),
        ));
    }
    match &app.phase {
        Phase::AwaitingPaint => layers.push(veil(
            context,
            Veil::Loading,
            overlays::loading_card(context, starting_progress()),
        )),
        Phase::Loading(progress) => layers.push(veil(
            context,
            Veil::Loading,
            overlays::loading_card(context, *progress),
        )),
        Phase::Failed(failure) => layers.push(veil(
            context,
            Veil::Error,
            overlays::error_card(context, failure, &app.fields.install_path),
        )),
        Phase::Ready => {}
    }
    // The body stays the first child of a stack even with no surface open, so opening or
    // closing a surface never changes the body's position in the widget tree and the grid
    // keeps its scroll offsets and focus.
    Stack::with_children(layers)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Progress shown before the first progress report arrives.
fn starting_progress() -> LoadProgress {
    LoadProgress {
        stage: Text::LoadStarting,
        fraction: 0.0,
    }
}

/// Transparent full-window layer under a popover menu: a press anywhere outside the menu,
/// including on the menu's own opener, closes it instead of reaching the control beneath.
fn outside_press<'a>() -> Element<'a, Message> {
    mouse_area(space().width(Length::Fill).height(Length::Fill))
        .on_press(Message::Overlay(OverlayMessage::Close))
        .into()
}

/// Content of a popover menu.
fn menu_content(app: &App, context: ViewContext, menu: Menu) -> Element<'_, Message> {
    match menu {
        Menu::Columns => column_menu::view(context, &app.columns),
        Menu::Language => settings::language_menu(context, app.locale),
        Menu::Appearance => settings::appearance_menu(context, &app.appearance),
    }
}

/// Places a menu at its anchor. Only the menu itself is opaque; the surrounding area lets
/// presses fall through to the outside-press layer.
fn popover<'a>(
    context: ViewContext,
    anchor: MenuAnchor,
    content: Element<'a, Message>,
) -> Element<'a, Message> {
    let menu = container(content)
        .width(anchor.width)
        .max_height(anchor.max_height);
    container(opaque(menu))
        .padding(end_padding(context.direction, anchor.top, anchor.end_inset))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(context.direction.end())
        .align_y(Vertical::Top)
        .into()
}

/// Padding that puts content `top` below the window top and `end_inset` from the inline end
/// edge; iced padding is physical, so the inline end is the right side only left to right.
#[must_use]
pub(super) fn end_padding(direction: Direction, top: f32, end_inset: f32) -> Padding {
    let padding = Padding::ZERO.top(top);
    match direction {
        Direction::Ltr => padding.right(end_inset),
        Direction::Rtl => padding.left(end_inset),
    }
}

/// Content of a narrow-layout drawer: the rail or the detail pane in its drawer form.
fn drawer_content(app: &App, context: ViewContext, drawer: Drawer) -> Element<'_, Message> {
    match drawer {
        Drawer::Filters => filter_rail::view(context, rail_model(app), false, true),
        Drawer::Details => detail::view(context, app.detail.as_ref(), true),
    }
}

/// Window-covering veil that centres its card and blocks pointer input to everything beneath.
fn veil<'a>(context: ViewContext, kind: Veil, card: Element<'a, Message>) -> Element<'a, Message> {
    let palette = context.palette;
    opaque(
        container(card)
            .center(Length::Fill)
            .style(move |_: &iced::Theme| theme::veil(palette, kind)),
    )
}
