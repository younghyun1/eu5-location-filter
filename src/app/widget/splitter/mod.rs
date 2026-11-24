//! Pane splitter: an `SPLITTER_HIT_WIDTH` drag target between a docked pane and the results,
//! drawing a `line` hairline at rest and an `accent` line while hovered, focused or dragged.
//! It publishes the requested pane width (start width plus signed pointer movement, unclamped)
//! and, while focused, one `PANE_KEY_STEP` per ArrowLeft or ArrowRight in the arrow's screen
//! direction.
//!
//! iced has no pointer capture, but every widget receives every cursor move, so a drag keeps
//! following the pointer after it leaves the 8 px strip until the button is released anywhere.
//! A pointer press focuses the splitter for the arrow keys without the focus-visible accent;
//! focus from a focus operation shows it.

mod handle;
mod state;
mod widget_impl;

pub(crate) use handle::Splitter;
