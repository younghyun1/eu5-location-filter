//! Selectors for the widgets the stock selectors cannot tell apart.
//!
//! Many catalog texts appear in several places at once: a column title is in the grid header and
//! in the column menu, an option label is in the rail and a cell value in the grid. Selectors
//! only see what widgets report from `operate`, so a text is chosen together with the bounds of
//! a region found by identifier.

use iced::Rectangle;
use iced_test::selector::{Candidate, Target};
use iced_test::{Error as SimulatorError, Selector};

use super::harness::{Ui, ui_error};
use crate::AppError;
use crate::app::ids;

/// Matches the first text with exactly `content` whose centre lies inside `region`.
pub(super) struct TextIn {
    content: String,
    region: Rectangle,
}

impl TextIn {
    /// A selector for `content` inside `region`.
    pub(super) fn new(content: impl Into<String>, region: Rectangle) -> Self {
        Self {
            content: content.into(),
            region,
        }
    }
}

impl Selector for TextIn {
    type Output = Target;

    fn select(&mut self, candidate: Candidate<'_>) -> Option<Target> {
        let hit = matches!(
            &candidate,
            Candidate::Text { content, bounds, .. }
                if *content == self.content.as_str() && self.region.contains(bounds.center())
        );
        hit.then(|| Target::from(candidate))
    }

    fn description(&self) -> String {
        format!("text == {:?} inside {:?}", self.content, self.region)
    }
}

/// Layout bounds of the widget reported under `id`.
pub(super) fn bounds_of(ui: &mut Ui<'_>, id: iced::widget::Id) -> Result<Rectangle, SimulatorError> {
    Ok(ui.find(id)?.bounds())
}

/// A selector for the text `content` inside the widget reported under `region`. Built apart
/// from the call that uses it, because finding the region borrows the simulator.
pub(super) fn text_selector(
    ui: &mut Ui<'_>,
    region: iced::widget::Id,
    content: &str,
) -> Result<TextIn, SimulatorError> {
    Ok(TextIn::new(content, bounds_of(ui, region)?))
}

/// The text `content` inside the widget reported under `region`.
pub(super) fn text_in(
    ui: &mut Ui<'_>,
    region: iced::widget::Id,
    content: &str,
) -> Result<Target, SimulatorError> {
    let selector = text_selector(ui, region, content)?;
    ui.find(selector)
}

/// The text `content` inside the result grid: a column title or a cell.
pub(super) fn grid_text(ui: &mut Ui<'_>, content: &str) -> Result<Target, SimulatorError> {
    text_in(ui, ids::GRID, content)
}

/// Whether the grid shows the text `content`.
pub(super) fn grid_shows(ui: &mut Ui<'_>, content: &str) -> bool {
    grid_text(ui, content).is_ok()
}

/// Whether a widget with identifier `id` is reported.
pub(super) fn has_id(ui: &mut Ui<'_>, id: iced::widget::Id) -> bool {
    ui.find(id).is_ok()
}

/// Vertical position of the grid text `content`; a smaller value is a higher row.
pub(super) fn row_y(ui: &mut Ui<'_>, content: &str) -> Result<f32, AppError> {
    grid_text(ui, content)
        .map(|target| target.bounds().y)
        .map_err(ui_error)
}
