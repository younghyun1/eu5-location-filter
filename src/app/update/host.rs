//! Browser host values and view-state reporting. Compiled on every target so the message type
//! stays target-independent; on desktop nothing produces these messages and reporting never
//! starts, because the host is never connected.

use iced::{Rectangle, Task};

use super::fields::sync_from_filters;
use super::filters::{filter, rebuild_detail, refresh_results};
use super::settings::{apply_reduced_motion, set_locale, set_palette};
use crate::app::message::{FilterMessage, HostSnapshot, Message};
use crate::app::state::{App, Phase};
use crate::app::view_model::view_state::{ViewSnapshot, decode, encode_into, resolve};

/// Applies drained host values: locale (with language and font pack loads), reduced motion,
/// palette (as a fade target), connection, and a view state, which is applied now when the
/// dataset and option lists are ready or kept in `HostState::pending_view_state` otherwise.
///
/// The order matches the host script's export order (locale, motion, view state, palette),
/// and `connected` is applied last, so a deep link is queued before reporting can start.
pub(crate) fn apply_host_snapshot(app: &mut App, snapshot: HostSnapshot) -> Task<Message> {
    let HostSnapshot {
        palette,
        locale,
        reduced_motion,
        view_state,
        connected,
        search_text,
    } = snapshot;
    let mut tasks = Vec::with_capacity(3);
    if let Some(locale) = locale {
        tasks.push(set_locale(app, locale));
    }
    if let Some(reduced) = reduced_motion {
        apply_reduced_motion(app, reduced);
    }
    if let Some(text) = view_state {
        app.host.pending_view_state = Some(text);
        tasks.push(apply_pending_view_state(app));
    }
    // Typed after any view state in the same drain, so it is the newer search.
    if let Some(text) = search_text {
        tasks.push(filter(app, FilterMessage::Search(text)));
    }
    if let Some(palette) = palette {
        set_palette(app, palette);
    }
    app.host.connected |= connected;
    Task::batch(tasks)
}

/// Stores the header search slot bounds published by the `Anchor` widget.
pub(super) fn search_anchor(app: &mut App, bounds: Rectangle) -> Task<Message> {
    let finite = [bounds.x, bounds.y, bounds.width, bounds.height]
        .iter()
        .all(|value| value.is_finite());
    // Reading the slot before writing keeps it in use on targets without a host page.
    if finite && app.host.search_anchor != Some(bounds) {
        app.host.search_anchor = Some(bounds);
    }
    Task::none()
}

/// Decodes and resolves the pending view state once data is ready, replacing the whole filter,
/// sort and selection state and resetting the rail fields to match.
pub(super) fn apply_pending_view_state(app: &mut App) -> Task<Message> {
    if !matches!(app.phase, Phase::Ready) {
        return Task::none();
    }
    let Some(data) = app.data.as_ref() else {
        return Task::none();
    };
    let Some(text) = app.host.pending_view_state.take() else {
        return Task::none();
    };
    let raw = match decode(&text) {
        Ok(raw) => raw,
        // A rejected string leaves the view unchanged; the next report replaces it upstream.
        Err(_rejected) => return Task::none(),
    };
    let resolved = resolve(&raw, &data.options, &data.location_keys, &data.dataset);
    app.filters = resolved.filters;
    app.sort = resolved.sort;
    app.selection = resolved.selected;
    sync_from_filters(&mut app.fields, &app.filters);
    // Keeps the resolved selection only if it is in the new results.
    refresh_results(app);
    rebuild_detail(app);
    Task::none()
}

/// When reporting is enabled (dataset ready and host connected), encodes the canonical string
/// and sends it only if it differs from `HostState::last_reported`. The host script debounces.
pub(super) fn report_view_state(app: &mut App) {
    let Some(state) = canonical_view_state(app) else {
        return;
    };
    if app.host.last_reported.as_deref() == Some(state.as_str()) {
        return;
    }
    send_report(&state);
    app.host.last_reported = Some(state);
}

/// Canonical view-state string of the current view, or `None` while reporting is disabled:
/// before the host connects, before the dataset is ready, and while a received view state is
/// still waiting to be applied, so a deep link is never overwritten by the default view.
pub(super) fn canonical_view_state(app: &App) -> Option<String> {
    let ready = matches!(app.phase, Phase::Ready);
    if !app.host.connected || !ready || app.host.pending_view_state.is_some() {
        return None;
    }
    let data = app.data.as_ref()?;
    let selected_key = app
        .selection
        .and_then(|id| data.dataset.location(id))
        .and_then(|record| data.dataset.symbol(record.key));
    let mut state = String::new();
    encode_into(
        &mut state,
        &ViewSnapshot {
            filters: &app.filters,
            sort: app.sort,
            selected_key,
            dataset: data.dataset.as_ref(),
        },
    );
    Some(state)
}

/// Sends a report to the host script.
#[cfg(all(feature = "web", target_family = "wasm"))]
fn send_report(state: &str) {
    crate::app::host::report(state);
}

/// Desktop has no host; reporting never becomes enabled there, so this is never reached.
#[cfg(not(all(feature = "web", target_family = "wasm")))]
fn send_report(state: &str) {
    let _unsent = state;
}

/// Bounds for the host page's search field, or `None` when it must be hidden: before the
/// dataset is ready (the loading or error veil covers the header), while the About dialog or
/// a drawer with its scrim is shown, and before the slot has been laid out.
#[cfg(any(test, all(feature = "web", target_family = "wasm")))]
pub(super) fn search_overlay_bounds(app: &App) -> Option<Rectangle> {
    use crate::app::state::Surface;

    let covered = !matches!(app.phase, Phase::Ready)
        || app.overlay.surface == Some(Surface::About)
        || app.panes.drawer.is_some();
    if covered {
        None
    } else {
        app.host.search_anchor
    }
}

/// Places, fills or hides the host page's search field after an update.
#[cfg(all(feature = "web", target_family = "wasm"))]
pub(super) fn sync_search_overlay(app: &App) {
    use crate::app::i18n::{Text, t};

    let overlay = search_overlay_bounds(app).map(|bounds| crate::app::host::SearchOverlay {
        bounds,
        text: &app.filters.search,
        placeholder: t(app.locale, Text::HeaderSearchPlaceholder),
        rtl: app.direction.is_rtl(),
    });
    crate::app::host::sync_search_overlay(overlay);
}
