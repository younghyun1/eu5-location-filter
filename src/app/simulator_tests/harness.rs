//! Shared plumbing of the simulator tests: an application with the synthetic dataset, a
//! simulator built over its current view, and the loop that applies the messages an interaction
//! published.
//!
//! A simulator holds one built interface that is never rebuilt, and it runs neither tasks nor
//! subscriptions. A test therefore interacts, collects the published messages, applies them
//! through `update`, and builds a new simulator to assert on the result. Focus and scroll state
//! live in the interface tree and are lost by that rebuild.

use iced::time::Duration;
use iced::{Settings, Size};
use iced_test::{Error as SimulatorError, Simulator};

use super::fixture;
use crate::AppError;
use crate::app::context::Platform;
use crate::app::i18n::{Locale, Text};
use crate::app::message::Message;
use crate::app::panes::controls::counted;
use crate::app::state::App;
use crate::app::theme::PALETTE_DARK;
use crate::app::update::update;
use crate::app::view::view;
use crate::model::LocationId;

/// Window size of the wide tests: both panes docked with room for several grid columns.
pub(super) const WIDE: Size = Size::new(1600.0, 900.0);
/// Window size of the narrow tests, below the layout breakpoint.
pub(super) const NARROW: Size = Size::new(600.0, 900.0);

/// The simulator type every test uses.
pub(super) type Ui<'a> = Simulator<'a, Message>;

/// Converts a simulator failure into the crate error, so tests return one error type.
pub(super) fn ui_error(error: SimulatorError) -> AppError {
    AppError::Ui(error.to_string())
}

/// An application and the helpers that drive it.
pub(super) struct Harness {
    /// The state under test.
    pub(super) app: App,
}

impl Harness {
    /// A ready English desktop application at `size`.
    pub(super) fn new(size: Size) -> Result<Self, AppError> {
        Self::with_locale(Locale::EnUs, size)
    }

    /// A ready desktop application in `locale` at `size`.
    pub(super) fn with_locale(locale: Locale, size: Size) -> Result<Self, AppError> {
        let app = App::with_data(Platform::Desktop, locale, PALETTE_DARK, fixture::loaded()?);
        Ok(Self::from_app(app, size))
    }

    /// Wraps an application and gives it the window size `size`.
    pub(super) fn from_app(app: App, size: Size) -> Self {
        let mut harness = Self { app };
        harness.send(Message::WindowResized(size));
        harness
    }

    /// Applies one message. The returned task is dropped: the simulator runs no tasks, and the
    /// tests that care about a task's effect assert the state change that precedes it.
    pub(super) fn send(&mut self, message: Message) {
        let _task = update(&mut self.app, message);
    }

    /// Applies each message in order.
    pub(super) fn apply(&mut self, messages: Vec<Message>) {
        for message in messages {
            self.send(message);
        }
    }

    /// Lets running animations finish by delivering a frame two seconds after the last one, so
    /// a drawer that just opened is fully drawn.
    pub(super) fn settle(&mut self) {
        let later = self.app.appearance.now + Duration::from_secs(2);
        self.send(Message::Frame(later));
    }

    /// A simulator over the current view, sized like the application's viewport.
    pub(super) fn ui(&self) -> Ui<'_> {
        Simulator::with_size(Settings::default(), self.app.viewport, view(&self.app))
    }

    /// Runs `script` against a fresh simulator and returns the messages it published, without
    /// applying them.
    pub(super) fn interact<F>(&self, script: F) -> Result<Vec<Message>, AppError>
    where
        F: FnOnce(&mut Ui<'_>) -> Result<(), SimulatorError>,
    {
        let mut ui = self.ui();
        script(&mut ui).map_err(ui_error)?;
        Ok(ui.into_messages().collect())
    }

    /// Runs `script` and applies the messages it published; returns them for inspection.
    pub(super) fn drive<F>(&mut self, script: F) -> Result<Vec<Message>, AppError>
    where
        F: FnOnce(&mut Ui<'_>) -> Result<(), SimulatorError>,
    {
        let messages = self.interact(script)?;
        self.apply(messages.clone());
        Ok(messages)
    }

    /// The current result identifiers, in grid order.
    pub(super) fn results(&self) -> Result<&[LocationId], AppError> {
        match self.app.data.as_ref() {
            Some(data) => Ok(data.results.as_slice()),
            None => Err(AppError::InvalidData("no dataset is installed".to_owned())),
        }
    }

    /// The header's result count text for `count`, built by the same function the header uses.
    pub(super) fn count_text(&self, count: usize) -> String {
        counted(self.app.view_context(), Text::HeaderResultCount, count)
    }
}
