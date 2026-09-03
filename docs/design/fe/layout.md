# Frontend layout

The Slint interface uses the compiled `fluent-dark` style. Shared colors and spacing remain in the `.slint` sources rather than Rust. `app.slint` owns the window, header, virtualized result list, detail pane, progress/error overlays, and About dialog. `filter-rail.slint` owns filter controls and forwards typed events to the window.

Rust converts the bounded dataset dictionary to shared UI strings once. The virtualized Slint model stores only filtered `LocationId` values and materializes rows only when Slint requests visible data; it does not expand every possible column for all 28,573 locations at startup. A filter scan emits one model reset. Embedded bundle loading, decompression, and index validation run on a worker thread and communicate through Slint's event-loop dispatcher.

The window declares preferred and minimum dimensions rather than a fixed width and height, so native resize, maximize, and fullscreen state remain under the window manager's control. Drag handles resize the filter rail, detail pane, and individual result columns. Each column boundary uses one visible pixel with a centered nine-pixel drag target. The result table scrolls horizontally when its configured columns exceed the viewport.

Categorical facets use reusable checkbox lists with a pinned search field and bounded list height. Each list is constrained to the current filter-rail viewport and inset from its clipping edge. Options show localized labels with internal IDs and match ASCII-folded input. River bounds select the five gameplay bonus tiers; only continuous numeric ranges and exact RGB accept free text.

The Columns popup exposes every filter criterion plus static population capacity and its equator contribution. Column headers display `^` or `v` for the active direction. Column visibility and widths update one bounded model shared by the header, rows, and popup. The result list owns both scroll axes, which keeps its vertical scrollbar at the visible pane edge when the table is wider than the pane.
