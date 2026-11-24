//! Local timings of the interface over the committed bundles: filter updates, view
//! construction with layout, scroll event handling, and a full offscreen frame.
//!
//! Ignored by default. Run with the optimized profile for meaningful numbers:
//! `cargo test --release --lib perf -- --ignored --nocapture --test-threads=1`. The frame time
//! includes offscreen rendering and pixel readback, so it is an upper bound on a displayed frame.

use std::hint::black_box;
use std::time::{Duration, Instant};

use iced::mouse::{self, ScrollDelta};
use iced::{Event, Point, Settings, Size};
use iced_test::Simulator;
use iced_test::core::renderer::Headless;
use iced_test::futures::futures::executor::block_on;
use iced_test::runtime::user_interface::{Cache, UserInterface};

use super::harness::{Harness, ui_error};
use crate::AppError;
use crate::app::context::Platform;
use crate::app::i18n::Locale;
use crate::app::load::prepare;
use crate::app::message::{FilterMessage, Message};
use crate::app::state::App;
use crate::app::theme::PALETTE_DARK;
use crate::app::view::{theme, view};

const SAMPLES: usize = 101;

fn summary(label: &str, mut samples: Vec<Duration>) {
    samples.sort_unstable();
    let median = samples.get(samples.len() / 2).copied().unwrap_or_default();
    let p95 = samples
        .get(samples.len() * 95 / 100)
        .copied()
        .unwrap_or_default();
    eprintln!(
        "{label}: median={median:?}, p95={p95:?}, samples={}",
        samples.len()
    );
}

fn real_app() -> Result<Harness, AppError> {
    let (dataset, index) = crate::embedded::load()?;
    let data = prepare(dataset, Some(index))?;
    let app = App::with_data(Platform::Desktop, Locale::EnUs, PALETTE_DARK, data);
    Ok(Harness::from_app(app, Size::new(1440.0, 900.0)))
}

#[test]
#[ignore = "local interface timings over the committed bundles"]
fn measures_interface_phases() -> Result<(), AppError> {
    let mut harness = real_app()?;
    eprintln!("rows in the default result: {}", harness.results()?.len());

    // Keystroke to updated state: one update per typed prefix, including the filter query,
    // selection preservation and option bookkeeping.
    let queries = [
        "a", "ab", "sto", "stock", "new", "saint", "", "ki", "kiev", "x",
    ];
    let mut updates = Vec::with_capacity(SAMPLES);
    for index in 0..SAMPLES {
        let query = queries[index % queries.len()].to_owned();
        let started = Instant::now();
        harness.send(Message::Filter(FilterMessage::Search(query)));
        updates.push(started.elapsed());
    }
    summary("update(search)", updates);
    harness.send(Message::Filter(FilterMessage::Search(String::new())));

    // What iced does after every message that reaches `update`: build the view, diff it
    // against the previous tree and lay it out. One renderer is reused, as in the running
    // application; the first build is excluded because it has no cache to diff against.
    let Some(mut renderer) = block_on(<iced::Renderer as Headless>::new(
        crate::app::theme::SANS,
        Settings::default().default_text_size,
        None,
    )) else {
        return Err(AppError::Ui("no headless renderer is available".to_owned()));
    };
    let size = harness.app.viewport;
    let mut cache = UserInterface::build(view(&harness.app), size, Cache::default(), &mut renderer)
        .into_cache();
    let mut views = Vec::with_capacity(SAMPLES);
    let mut rebuilds = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let started = Instant::now();
        let element = view(&harness.app);
        views.push(started.elapsed());
        let interface = UserInterface::build(element, size, cache, &mut renderer);
        rebuilds.push(started.elapsed());
        cache = interface.into_cache();
    }
    drop(cache);
    summary("view()", views);
    summary("view() + diff + layout", rebuilds);

    // Wheel scrolling over the grid: event handling only, on one built interface.
    let mut ui: Simulator<'_, Message> = Simulator::with_size(
        Settings::default(),
        harness.app.viewport,
        view(&harness.app),
    );
    ui.point_at(Point::new(760.0, 520.0));
    let mut scrolls = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let wheel = Event::Mouse(mouse::Event::WheelScrolled {
            delta: ScrollDelta::Lines { x: 0.0, y: -3.0 },
        });
        let started = Instant::now();
        let _ = black_box(ui.simulate([wheel]));
        scrolls.push(started.elapsed());
    }
    summary("scroll event", scrolls);

    // A full offscreen frame after each scroll step: draw, render and read back.
    let palette_theme = theme(&harness.app);
    let mut frames = Vec::with_capacity(31);
    for _ in 0..31 {
        let wheel = Event::Mouse(mouse::Event::WheelScrolled {
            delta: ScrollDelta::Lines { x: 0.0, y: -3.0 },
        });
        let _ = ui.simulate([wheel]);
        let started = Instant::now();
        let snapshot = ui.snapshot(&palette_theme).map_err(ui_error)?;
        frames.push(started.elapsed());
        drop(black_box(snapshot));
    }
    summary(
        "offscreen frame (draw + render + readback, 2x scale)",
        frames,
    );
    Ok(())
}
