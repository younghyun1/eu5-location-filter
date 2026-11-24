//! Menus, dialogs, drawers, notices and links, with the opener bookkeeping that returns
//! keyboard focus when a surface closes.

use iced::Task;
use iced::time::Instant;
use iced::widget::{Id, operation};

use crate::app::ids;
use crate::app::layout::{Drawer, LayoutMode};
use crate::app::links;
use crate::app::message::{Message, OverlayMessage};
use crate::app::state::{App, Surface, drawer_animation};

/// Opens and closes menus, dialogs and drawers (recording the opener for focus return),
/// toggles licence texts, dismisses notices and opens links.
pub(super) fn overlay(app: &mut App, message: OverlayMessage) -> Task<Message> {
    match message {
        OverlayMessage::Open(surface) => open_surface(app, surface),
        OverlayMessage::Toggle(surface) => {
            if app.overlay.surface == Some(surface) {
                close_surface(app)
            } else {
                open_surface(app, surface)
            }
        }
        OverlayMessage::Close => close_topmost(app),
        OverlayMessage::OpenDrawer(drawer) => open_drawer(app, drawer),
        OverlayMessage::CloseDrawer => close_drawer(app),
        OverlayMessage::SetLicensesExpanded(expanded) => {
            app.overlay.licenses_expanded = expanded;
            Task::none()
        }
        OverlayMessage::DismissNotice => {
            app.overlay.notice = None;
            Task::none()
        }
        OverlayMessage::OpenLink(link) => {
            // A link that cannot be opened has no surface to report on; the button stays
            // available, so the user can try again.
            let _unopened = links::open(link);
            Task::none()
        }
    }
}

/// Opener of each surface, focused again when it closes.
fn opener(surface: Surface) -> Id {
    match surface {
        Surface::ColumnMenu => ids::COLUMNS_BUTTON,
        Surface::LanguageMenu => ids::LANGUAGE_BUTTON,
        Surface::AppearanceMenu => ids::APPEARANCE_BUTTON,
        Surface::About => ids::ABOUT_BUTTON,
    }
}

/// Opens one surface, replacing any other; desktop menus do not exist in the browser.
fn open_surface(app: &mut App, surface: Surface) -> Task<Message> {
    let desktop_only = matches!(surface, Surface::LanguageMenu | Surface::AppearanceMenu);
    if desktop_only && !app.platform.has_desktop_controls() {
        return Task::none();
    }
    if surface == Surface::About {
        app.overlay.licenses_expanded = false;
    }
    app.overlay.surface = Some(surface);
    app.focus.return_to = Some(opener(surface));
    Task::none()
}

/// Closes the open surface and focuses its opener.
fn close_surface(app: &mut App) -> Task<Message> {
    if app.overlay.surface.take().is_none() {
        return Task::none();
    }
    return_focus(app)
}

/// Closes the topmost surface: a menu or dialog above an open drawer.
pub(super) fn close_topmost(app: &mut App) -> Task<Message> {
    if app.overlay.surface.is_some() {
        close_surface(app)
    } else if app.open_drawer().is_some() {
        close_drawer(app)
    } else {
        Task::none()
    }
}

/// Opens a narrow-layout drawer and starts its slide; ignored in the wide layout, where the
/// panes are docked.
pub(super) fn open_drawer(app: &mut App, drawer: Drawer) -> Task<Message> {
    if app.layout_mode != LayoutMode::Narrow {
        return Task::none();
    }
    // The subscription reads `appearance.now` to decide whether frames are needed, so the
    // clock moves to the start of the slide.
    let now = Instant::now();
    app.appearance.now = now;
    app.overlay.surface = None;
    app.panes.drawer = Some(drawer);
    app.overlay.drawer_motion.go_mut(true, now);
    app.focus.return_to = Some(match drawer {
        Drawer::Filters => ids::FILTERS_BUTTON,
        Drawer::Details => ids::DETAILS_BUTTON,
    });
    Task::none()
}

/// Starts closing the drawer and focuses its opener. The drawer stays in `PaneState::drawer`
/// until the slide ends, so the view can draw it sliding out; without motion it goes at once.
fn close_drawer(app: &mut App) -> Task<Message> {
    if app.panes.drawer.is_none() {
        return Task::none();
    }
    let now = Instant::now();
    app.appearance.now = now;
    app.overlay.drawer_motion.go_mut(false, now);
    if app.appearance.reduced_motion || !app.overlay.drawer_motion.is_animating(now) {
        app.panes.drawer = None;
    }
    return_focus(app)
}

/// Removes the drawer without a slide, for a switch to the wide layout.
pub(super) fn dismiss_drawer(app: &mut App) {
    if app.panes.drawer.take().is_some() {
        app.overlay.drawer_motion = drawer_animation(false, app.appearance.reduced_motion);
        app.focus.return_to = None;
    }
}

/// Focuses the recorded opener, if any.
fn return_focus(app: &mut App) -> Task<Message> {
    match app.focus.return_to.take() {
        Some(id) => operation::focus(id),
        None => Task::none(),
    }
}
