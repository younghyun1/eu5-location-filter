//! The application state, its loaded-data part and its constructors.

use std::path::PathBuf;
use std::sync::Arc;

use iced::Size;

use super::{Appearance, FocusState, OverlayState, PackState, Phase, TextFields};
use crate::app::context::{Platform, ViewContext};
use crate::app::i18n::Locale;
use crate::app::layout::{self, Direction, LayoutMode, PaneState};
use crate::app::load::LoadedData;
use crate::app::theme::Palette;
use crate::app::view_model::{
    Columns, FacetId, FacetOptions, FacetUi, LocationDetail, LocationKeys, SortState,
};
use crate::filter::{FilterEngine, FilterSet};
use crate::lang_pack::Labels;
use crate::model::{Dataset, LocationId};

/// Desktop command-line paths, kept for retry and rebuild. Empty in the browser.
#[derive(Clone, Debug, Default)]
pub(crate) struct LaunchPaths {
    /// External location bundle; `None` loads the embedded one.
    pub(crate) data_file: Option<PathBuf>,
    /// External index bundle; `None` infers it from `data_file`.
    pub(crate) index_file: Option<PathBuf>,
    /// Game directory for import and rebuild; `None` uses Steam discovery.
    pub(crate) game_dir: Option<PathBuf>,
}

/// Everything that exists only once a dataset is loaded.
pub(crate) struct DataState {
    /// Immutable dataset shared with the engine and background pack validation.
    pub(crate) dataset: Arc<Dataset>,
    /// Owned by value because installing a language pack takes `&mut self`.
    pub(crate) engine: FilterEngine,
    /// Current filtered and sorted result IDs; the only copy, borrowed by the grid.
    pub(crate) results: Vec<LocationId>,
    /// Checklist option lists with labels folded for the active game language.
    pub(crate) options: FacetOptions,
    /// Location key lookup for view-state selection, built once per dataset.
    pub(crate) location_keys: LocationKeys,
}

/// Browser host connection and view-state reporting.
#[derive(Clone, Debug, Default)]
pub(crate) struct HostState {
    /// Set by the first `set_palette`; reporting starts only when this and the dataset are ready.
    pub(crate) connected: bool,
    /// Last canonical view-state string sent to the host, compared before each report.
    pub(crate) last_reported: Option<String>,
    /// View-state string received before the dataset and option lists were ready.
    pub(crate) pending_view_state: Option<String>,
}

/// The whole interface state. Fields are crate-visible so update helpers and panes read them
/// directly; only update helpers write them.
pub(crate) struct App {
    /// Running build.
    pub(crate) platform: Platform,
    /// Desktop command-line paths.
    pub(crate) launch: LaunchPaths,
    /// Loading, failure or ready.
    pub(crate) phase: Phase,
    /// Loaded dataset, engine, results and option lists.
    pub(crate) data: Option<DataState>,
    /// Active filters; `filters.search` is also the header field value.
    pub(crate) filters: FilterSet,
    /// Active sort.
    pub(crate) sort: SortState,
    /// Selected location, kept only while it stays in the results.
    pub(crate) selection: Option<LocationId>,
    /// Detail pane content for the selection, rebuilt on selection, locale or pack change.
    pub(crate) detail: Option<LocationDetail>,
    /// Column widths and visibility.
    pub(crate) columns: Columns,
    /// Stored pane widths, collapse and hide flags, and the open drawer.
    pub(crate) panes: PaneState,
    /// Fold state, option query and visible option indices per facet group, in `FacetId::ALL`
    /// order.
    pub(crate) facets: [FacetUi; FacetId::COUNT],
    /// Rail text field buffers with their inline errors, and the install path field.
    pub(crate) fields: TextFields,
    /// Open menu or dialog, notice, licence expansion and drawer animation.
    pub(crate) overlay: OverlayState,
    /// Focus region and the opener to return focus to.
    pub(crate) focus: FocusState,
    /// Palette, fade, theme, accent and motion.
    pub(crate) appearance: Appearance,
    /// Interface locale.
    pub(crate) locale: Locale,
    /// Layout direction of `locale`.
    pub(crate) direction: Direction,
    /// Logical viewport size from the last resize event.
    pub(crate) viewport: Size,
    /// Wide or narrow layout for `viewport`.
    pub(crate) layout_mode: LayoutMode,
    /// Active, pending and failed language and font packs.
    pub(crate) packs: PackState,
    /// Browser host connection and reporting state.
    pub(crate) host: HostState,
    /// Incremented for each data load so stale progress and results are ignored.
    pub(crate) load_generation: u64,
}

impl App {
    /// State before any data is loaded.
    #[must_use]
    pub(crate) fn new(
        platform: Platform,
        launch: LaunchPaths,
        locale: Locale,
        palette: Palette,
    ) -> Self {
        let viewport = Size::new(
            layout::DESKTOP_DEFAULT_WIDTH,
            layout::DESKTOP_DEFAULT_HEIGHT,
        );
        Self {
            platform,
            launch,
            phase: Phase::AwaitingPaint,
            data: None,
            filters: FilterSet::land_only(),
            sort: SortState::default(),
            selection: None,
            detail: None,
            columns: Columns::default(),
            panes: PaneState::default(),
            facets: FacetId::ALL.map(FacetUi::new),
            fields: TextFields::default(),
            overlay: OverlayState::default(),
            focus: FocusState::default(),
            appearance: Appearance::new(palette),
            locale,
            direction: locale.direction(),
            viewport,
            layout_mode: layout::mode_for_width(viewport.width),
            packs: PackState::default(),
            host: HostState::default(),
            load_generation: 0,
        }
    }

    /// Ready state from an already prepared dataset; used by simulator tests and benchmarks.
    #[must_use]
    pub(crate) fn with_data(
        platform: Platform,
        locale: Locale,
        palette: Palette,
        data: LoadedData,
    ) -> Self {
        let mut app = Self::new(platform, LaunchPaths::default(), locale, palette);
        app.install_data(data);
        app
    }

    /// Installs a prepared dataset: builds option lists for the active language, location key
    /// lookup and the first result set, and switches to `Phase::Ready`.
    pub(crate) fn install_data(&mut self, data: LoadedData) {
        let LoadedData { dataset, engine } = data;
        let options = FacetOptions::build(Labels::new(&dataset, None), self.locale);
        let location_keys = LocationKeys::build(&dataset);
        let results = engine.apply(&self.filters, self.sort.field, self.sort.ascending);
        self.data = Some(DataState {
            dataset,
            engine,
            results,
            options,
            location_keys,
        });
        self.phase = Phase::Ready;
    }

    /// View inputs for this frame.
    #[must_use]
    pub(crate) fn view_context(&self) -> ViewContext {
        ViewContext {
            palette: self.appearance.palette,
            locale: self.locale,
            direction: self.direction,
            mode: self.layout_mode,
            platform: self.platform,
        }
    }
}
