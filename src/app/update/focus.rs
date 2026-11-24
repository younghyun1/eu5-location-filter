//! Focus moves the runtime does not provide: Tab scoped to an open surface, region targets
//! for F6, focus of the search field, and the per-target search focus.

use iced::Task;
use iced::advanced::widget::operate;
use iced::advanced::widget::operation::{focusable, scope};
use iced::widget::{Id, operation};

use crate::app::ids;
use crate::app::layout::LayoutMode;
use crate::app::message::Message;
use crate::app::state::{App, FocusRegion};

/// Regions in F6 order.
pub(super) const REGIONS: [FocusRegion; 4] = [
    FocusRegion::Header,
    FocusRegion::Rail,
    FocusRegion::Grid,
    FocusRegion::Detail,
];

/// What F6 focuses in a region.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Target {
    /// The header search field: an iced text input on desktop, the host page's field in the
    /// browser.
    Search,
    /// A focusable widget.
    Widget(Id),
}

/// Whether a menu, dialog or drawer is open, in which case Tab stays inside it.
pub(super) fn surface_open(app: &App) -> bool {
    app.overlay.surface.is_some() || app.open_drawer().is_some()
}

/// Tab or Shift+Tab: next or previous focusable, limited to the container with
/// `ids::OPEN_SURFACE` while a surface is open, so focus wraps inside it.
pub(super) fn traverse(app: &App, forward: bool) -> Task<Message> {
    match (surface_open(app), forward) {
        (true, true) => operate(scope(ids::OPEN_SURFACE, focusable::focus_next::<()>())).discard(),
        (true, false) => {
            operate(scope(ids::OPEN_SURFACE, focusable::focus_previous::<()>())).discard()
        }
        (false, true) => operation::focus_next(),
        (false, false) => operation::focus_previous(),
    }
}

/// First control of a region in the current layout, or `None` when the region is not shown:
/// the narrow layout has no docked rail or detail pane, and a hidden detail pane has no
/// controls. A collapsed rail keeps its expand button, which shares the collapse button's id.
pub(super) fn region_target(app: &App, region: FocusRegion) -> Option<Target> {
    let narrow = app.layout_mode == LayoutMode::Narrow;
    match region {
        FocusRegion::Header if narrow => Some(Target::Widget(ids::FILTERS_BUTTON)),
        FocusRegion::Header => Some(Target::Search),
        FocusRegion::Rail if narrow => None,
        FocusRegion::Rail => Some(Target::Widget(ids::RAIL_COLLAPSE)),
        FocusRegion::Grid => Some(Target::Widget(ids::COLUMNS_BUTTON)),
        FocusRegion::Detail if narrow || app.panes.detail_hidden => None,
        FocusRegion::Detail => Some(Target::Widget(ids::DETAIL_HIDE)),
    }
}

/// Next or previous region after `from` that has a target, wrapping around.
pub(super) fn next_region(
    app: &App,
    from: FocusRegion,
    forward: bool,
) -> Option<(FocusRegion, Target)> {
    let count = REGIONS.len();
    let start = REGIONS.iter().position(|region| *region == from).unwrap_or(0);
    (1..=count).find_map(|step| {
        let offset = if forward { step } else { count - step };
        let region = *REGIONS.get((start + offset) % count)?;
        region_target(app, region).map(|target| (region, target))
    })
}

/// Task that focuses a target.
pub(super) fn focus_target(target: Target) -> Task<Message> {
    match target {
        Target::Search => focus_search(),
        Target::Widget(id) => operation::focus(id),
    }
}

/// Focuses the search field and selects its text.
#[cfg(not(all(feature = "web", target_family = "wasm")))]
pub(super) fn focus_search() -> Task<Message> {
    operation::focus(ids::SEARCH).chain(operation::select_all(ids::SEARCH))
}

/// Focuses the host page's search field, which selects its text; the canvas holds no search
/// input in the browser, because the host field is the one that supports input methods.
#[cfg(all(feature = "web", target_family = "wasm"))]
pub(super) fn focus_search() -> Task<Message> {
    crate::app::host::focus_search_overlay();
    Task::none()
}
