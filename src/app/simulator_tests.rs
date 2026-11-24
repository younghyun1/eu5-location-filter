//! Headless interface tests: the real `view` and `update` driven through `iced_test` over a
//! small synthetic dataset.
//!
//! Each test builds a simulator over the application's current view, interacts through widget
//! identifiers and reported texts, applies the published messages, and builds a new simulator
//! to assert on the result. Stock widgets and the custom widgets report what the tests find
//! through `operate`; checked state, selection and sort direction are not reported, so those
//! are asserted on `App` instead.

mod facets;
mod fixture;
mod harness;
mod keys;
mod narrow;
mod overlays;
mod panes;
mod search;
mod selectors;
mod table;
