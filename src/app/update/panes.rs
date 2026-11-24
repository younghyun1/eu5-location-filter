//! Pane geometry and window resizes.

use iced::{Size, Task};

use super::surfaces::dismiss_drawer;
use crate::app::layout::{self, LayoutMode, clamp_detail, clamp_rail, mode_for_width};
use crate::app::message::{Message, PaneMessage};
use crate::app::state::App;

/// Splitter widths (clamped), rail collapse and detail hide; showing the detail pane collapses
/// the rail when both cannot fit beside `GRID_MIN_WIDTH`.
pub(super) fn pane(app: &mut App, message: PaneMessage) -> Task<Message> {
    match message {
        PaneMessage::ResizeRail(width) => {
            // `f32::clamp` passes NaN through, so a non-finite width is dropped here.
            if width.is_finite() {
                app.panes.rail_width = clamp_rail(width);
                settle_layout(app);
            }
        }
        PaneMessage::ResizeDetail(width) => {
            if width.is_finite() {
                app.panes.detail_width = clamp_detail(width);
                settle_layout(app);
            }
        }
        PaneMessage::SetRailCollapsed(collapsed) => {
            app.panes.rail_collapsed = collapsed;
            settle_layout(app);
        }
        PaneMessage::SetDetailHidden(true) => app.panes.detail_hidden = true,
        PaneMessage::SetDetailHidden(false) => show_detail_pane(app),
    }
    Task::none()
}

/// Shows the detail pane. When the resolved wide layout cannot keep the grid at its minimum
/// beside both panes, the rail collapses so the pane the user asked for stays visible.
pub(super) fn show_detail_pane(app: &mut App) {
    app.panes.detail_hidden = false;
    if app.layout_mode != LayoutMode::Wide {
        return;
    }
    let resolved = layout::resolve(app.viewport.width, &app.panes, app.layout_mode);
    if resolved.detail_forced_hidden {
        app.panes.rail_collapsed = true;
        settle_layout(app);
    }
}

/// Stores the outcome of the wide-layout resolution: when both panes at their minimum leave
/// the grid too narrow, the detail pane is hidden, and it stays hidden until shown again.
pub(super) fn settle_layout(app: &mut App) {
    if app.layout_mode != LayoutMode::Wide {
        return;
    }
    let resolved = layout::resolve(app.viewport.width, &app.panes, app.layout_mode);
    if resolved.detail_forced_hidden {
        app.panes.detail_hidden = true;
    }
}

/// Stores the viewport and recomputes the layout mode; leaving the narrow layout closes any
/// drawer.
pub(super) fn resized(app: &mut App, size: Size) -> Task<Message> {
    if !(size.width.is_finite() && size.height.is_finite()) {
        return Task::none();
    }
    app.viewport = Size::new(size.width.max(0.0), size.height.max(0.0));
    let mode = mode_for_width(app.viewport.width);
    let previous = std::mem::replace(&mut app.layout_mode, mode);
    if previous == LayoutMode::Narrow && mode == LayoutMode::Wide {
        dismiss_drawer(app);
    }
    settle_layout(app);
    Task::none()
}
