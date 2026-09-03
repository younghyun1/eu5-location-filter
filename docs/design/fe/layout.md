# Frontend layout

The Slint interface uses the compiled `fluent-dark` style. Shared colors and spacing remain in the `.slint` sources rather than Rust. `app.slint` owns the window, header, virtualized result list, detail pane, progress/error overlays, and About dialog. `filter-rail.slint` owns filter controls and forwards typed events to the window.

Rust prebuilds immutable display rows once after loading. The custom Slint model stores only filtered `LocationId` values and emits one reset notification after a scan; filtering does not rebuild display strings. The import and blob-load path runs on a worker thread and communicates through Slint's event-loop dispatcher.

The window declares preferred and minimum dimensions rather than a fixed width and height, so native resize, maximize, and fullscreen state remain under the window manager's control. Drag handles resize the filter rail, detail pane, and individual result columns. The result table scrolls horizontally when its configured columns exceed the viewport.

Categorical facets use a reusable popup with a search field outside the scrolling option list, so search remains pinned at the top. Options show localized labels with internal IDs and match ASCII-folded input. River-level bounds are dropdowns; only continuous numeric ranges and exact RGB accept free text.

The Columns popup exposes every filter criterion as an optional result column. Column headers display `^` or `v` for the active direction. Column visibility and widths update one bounded model shared by the header, rows, and popup.
