//! Panes built from stock iced widgets: header, results toolbar, filter rail, detail pane,
//! column menu, About dialog, loading and error overlays, drawers, notices, and the desktop
//! settings menus. Each exposes one `view` function taking a `ViewContext` and exactly the data
//! it shows; none reads `App` directly.

pub(crate) mod about;
pub(crate) mod column_menu;
pub(crate) mod controls;
pub(crate) mod detail;
pub(crate) mod filter_rail;
pub(crate) mod header;
pub(crate) mod overlays;
pub(crate) mod settings;
pub(crate) mod toolbar;
