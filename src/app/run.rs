//! Application entry points for the desktop window and the browser canvas, and the boot
//! functions iced calls once to build the initial state.

#[cfg(feature = "desktop")]
use std::path::PathBuf;

use iced::Task;

use super::context::Platform;
use super::i18n::Locale;
use super::message::Message;
use super::state::{App, LaunchPaths};
use super::theme::PALETTE_DARK;
use super::{load, subscription, update, view};
use crate::AppError;

/// Opens the desktop window and starts loading the embedded bundles, or the external data and
/// game directory given on the command line, on a background thread.
#[cfg(feature = "desktop")]
pub fn run(
    data_file: Option<PathBuf>,
    index_file: Option<PathBuf>,
    game_dir: Option<PathBuf>,
) -> Result<(), AppError> {
    use iced::Size;

    use super::layout::{
        DESKTOP_DEFAULT_HEIGHT, DESKTOP_DEFAULT_WIDTH, DESKTOP_MIN_HEIGHT, DESKTOP_MIN_WIDTH,
    };

    let launch = LaunchPaths {
        data_file,
        index_file,
        game_dir,
    };
    let window = iced::window::Settings {
        size: Size::new(DESKTOP_DEFAULT_WIDTH, DESKTOP_DEFAULT_HEIGHT),
        min_size: Some(Size::new(DESKTOP_MIN_WIDTH, DESKTOP_MIN_HEIGHT)),
        ..iced::window::Settings::default()
    };
    let mut application = iced::application(
        move || boot_desktop(launch.clone()),
        update::update,
        view::view,
    )
    .title(view::title)
    .theme(view::theme)
    .style(view::window_style)
    .subscription(subscription::subscription)
    .default_font(super::theme::SANS)
    .antialiasing(false)
    .window(window);
    for face in load::embedded_faces() {
        application = application.font(face);
    }
    application
        .run()
        .map_err(|error| AppError::Ui(error.to_string()))
}

/// Starts the application in the browser; iced replaces the `<canvas id="canvas">` placeholder
/// in `web/index.html` with its own canvas and returns once the event loop is handed to the
/// browser.
#[cfg(all(feature = "web", target_family = "wasm"))]
pub(crate) fn run_web() -> Result<(), AppError> {
    let window = iced::window::Settings {
        platform_specific: iced::window::settings::PlatformSpecific {
            target: Some(String::from("canvas")),
        },
        ..iced::window::Settings::default()
    };
    let mut application = iced::application(boot_web, update::update, view::view)
        .title(view::title)
        .theme(view::theme)
        .style(view::window_style)
        .subscription(subscription::subscription)
        .default_font(super::theme::SANS)
        .antialiasing(false)
        .window(window);
    for face in load::embedded_faces() {
        application = application.font(face);
    }
    application
        .run()
        .map_err(|error| AppError::Ui(error.to_string()))
}

/// Desktop boot: dark amber palette, English until the user picks a language, and the first
/// background load.
#[cfg(feature = "desktop")]
fn boot_desktop(launch: LaunchPaths) -> (App, Task<Message>) {
    let app = App::new(Platform::Desktop, launch, Locale::EnUs, PALETTE_DARK);
    let task = load::start(&app.launch, app.load_generation, false);
    (app, task)
}

/// Browser boot: applies any host values that arrived before boot so the first frame already
/// uses them, then waits for the first painted frame before decoding the embedded bundles.
#[cfg(all(feature = "web", target_family = "wasm"))]
fn boot_web() -> (App, Task<Message>) {
    let mut app = App::new(
        Platform::Browser,
        LaunchPaths::default(),
        Locale::EnUs,
        PALETTE_DARK,
    );
    let task = update::apply_host_snapshot(&mut app, super::host::take_snapshot());
    (app, task)
}
