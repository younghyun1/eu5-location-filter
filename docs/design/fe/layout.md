# Frontend layout

The Slint interface uses the compiled `fluent-dark` style. Shared colors and spacing remain in the `.slint` sources rather than Rust. `app.slint` owns the window, header, virtualized result list, detail pane, progress/error overlays, and About dialog. `filter-rail.slint` owns filter controls and forwards typed events to the window.

Rust prebuilds immutable display rows once after loading. The custom Slint model stores only filtered `LocationId` values and emits one reset notification after a scan; filtering does not rebuild display strings. The import and blob-load path runs on a worker thread and communicates through Slint's event-loop dispatcher.
