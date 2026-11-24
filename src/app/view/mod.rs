//! Root view: composes header, body panes, toolbar, grid and overlays for the current
//! direction and layout mode, plus the window title, theme and window style callbacks.

mod body;
mod plan;
mod root;
mod surfaces;
#[cfg(test)]
mod tests;

pub(crate) use root::{theme, title, view, window_style};
