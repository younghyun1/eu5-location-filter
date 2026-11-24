//! Application keyboard commands: focus traversal, region cycling, search focus and Escape.

use iced::Task;

use super::focus::{self, focus_target, next_region, surface_open};
use super::surfaces::close_topmost;
use crate::app::message::{KeyCommand, Message};
use crate::app::state::{App, FocusRegion, Phase, Surface};

/// Tab and Shift+Tab return `focus_next` or `focus_previous` (wrapping inside an open surface),
/// Ctrl+F focuses and selects the search, F6 cycles regions, Escape closes the topmost surface
/// and focuses its opener.
pub(super) fn key(app: &mut App, command: KeyCommand) -> Task<Message> {
    match command {
        KeyCommand::FocusNext => focus::traverse(app, true),
        KeyCommand::FocusPrevious => focus::traverse(app, false),
        KeyCommand::FocusSearch => focus_search(app),
        KeyCommand::NextRegion => cycle_region(app, true),
        KeyCommand::PreviousRegion => cycle_region(app, false),
        KeyCommand::Escape => close_topmost(app),
    }
}

/// Ctrl+F. Ignored while the search is covered (loading or error veil, About dialog, drawer);
/// a non-modal menu closes first, because focus is leaving it.
fn focus_search(app: &mut App) -> Task<Message> {
    let covered = !matches!(app.phase, Phase::Ready)
        || app.overlay.surface == Some(Surface::About)
        || app.open_drawer().is_some();
    if covered {
        return Task::none();
    }
    if app.overlay.surface.take().is_some() {
        app.focus.return_to = None;
    }
    app.focus.region = FocusRegion::Header;
    focus::focus_search()
}

/// F6 and Shift+F6: first control of the next or previous shown region. An open surface keeps
/// focus, as Tab does.
fn cycle_region(app: &mut App, forward: bool) -> Task<Message> {
    if surface_open(app) {
        return Task::none();
    }
    match next_region(app, app.focus.region, forward) {
        Some((region, target)) => {
            app.focus.region = region;
            focus_target(target)
        }
        None => Task::none(),
    }
}
