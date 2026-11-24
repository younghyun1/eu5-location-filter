//! Desktop loading on dedicated threads reporting through channels: embedded bundles, external
//! bundles, import and rebuild (the previous worker's logic), and embedded language and font
//! packs. Thread spawn failures are reported as load failures, never unwrapped.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use iced::Task;
use iced::futures::channel::{mpsc, oneshot};

use super::external::load_or_import;
use super::packs::{font_message, register_font, validate_pack_bytes};
use super::{LoadedData, embedded_pack, prepare};
use crate::app::handoff::Handoff;
use crate::app::i18n::{FontPack, Text};
use crate::app::message::{LoadMessage, Message};
use crate::app::state::LaunchPaths;
use crate::lang_pack::{GameLanguage, ValidatedLanguagePack};
use crate::model::Dataset;
use crate::{AppError, embedded};

/// Events buffered between the loader thread and the interface. Progress beyond this is
/// dropped; the final result waits for room.
const EVENT_CAPACITY: usize = 16;
/// Pause between attempts to queue the final result when the buffer is full.
const RESULT_RETRY: Duration = Duration::from_millis(5);
/// Fraction shown while the engine is restored or built after the bundles are decoded.
const PREPARE_FRACTION: f32 = 0.9;

/// What the loader thread reports.
enum LoadEvent {
    /// A stage began or advanced.
    Progress {
        /// Catalog text for the stage.
        stage: Text,
        /// Completed fraction.
        fraction: f32,
    },
    /// The load ended; the last event of a load.
    Done(Result<Handoff<LoadedData>, Arc<AppError>>),
}

/// Starts a load with this generation: embedded bundles when no data file is given, the
/// external bundle and its index when present, otherwise (or when `force`) an import from the
/// game directory that writes the external bundles. Progress arrives as
/// `LoadMessage::Progress` through a bounded channel whose overflow drops progress, never the
/// result, which arrives once as `LoadMessage::Finished`.
pub(crate) fn start(launch: &LaunchPaths, generation: u64, force: bool) -> Task<Message> {
    let launch = launch.clone();
    let (sender, receiver) = mpsc::channel(EVENT_CAPACITY);
    let spawned = thread::Builder::new()
        .name("eu5-load".to_owned())
        .spawn(move || run_load(&launch, force, sender));
    match spawned {
        Ok(_detached) => Task::run(receiver, move |event| event_message(generation, event)),
        Err(source) => Task::done(Message::Load(LoadMessage::Finished {
            generation,
            result: Err(Arc::new(AppError::io("start", "eu5-load thread", source))),
        })),
    }
}

/// Decodes and validates the embedded pack for `language` on a thread and answers with
/// `LoadMessage::LanguagePackValidated`; a language without an embedded pack fails with
/// `InvalidData`.
pub(crate) fn language_pack(language: GameLanguage, dataset: &Arc<Dataset>) -> Task<Message> {
    let dataset = Arc::clone(dataset);
    on_thread("eu5-language-pack", move || {
        validate_embedded(language, &dataset)
    })
    .map(move |outcome| {
        let result = match outcome {
            Ok(Ok(pack)) => Ok(Handoff::new(pack)),
            Ok(Err(error)) | Err(error) => Err(Arc::new(error)),
        };
        Message::Load(LoadMessage::LanguagePackValidated { language, result })
    })
}

/// Registers the embedded script pack with `iced::font::load` and answers with
/// `LoadMessage::FontPackLoaded`.
pub(crate) fn font_pack(pack: FontPack) -> Task<Message> {
    match embedded_pack(pack) {
        Some(bytes) => register_font(pack, bytes),
        None => Task::done(font_message(
            pack,
            Err(AppError::InvalidData(format!(
                "font pack {} is not embedded in this build",
                pack.file_stem()
            ))),
        )),
    }
}

/// Validates the embedded pack bytes of `language` against `dataset`.
fn validate_embedded(
    language: GameLanguage,
    dataset: &Dataset,
) -> Result<ValidatedLanguagePack, AppError> {
    match embedded::language_pack(language) {
        Some(bytes) => validate_pack_bytes(bytes, dataset, language),
        None => Err(AppError::InvalidData(format!(
            "no embedded language pack for {}",
            language.code()
        ))),
    }
}

/// Runs `work` on a named thread. A spawn failure or a panic in `work` becomes an error, so the
/// returned task always produces exactly one value.
fn on_thread<T: Send + 'static>(
    name: &'static str,
    work: impl FnOnce() -> T + Send + 'static,
) -> Task<Result<T, AppError>> {
    let (sender, receiver) = oneshot::channel();
    let spawned = thread::Builder::new().name(name.to_owned()).spawn(move || {
        // A panic drops the sender unsent, which the receiving side reports as `Worker`.
        if let Ok(value) = catch_unwind(AssertUnwindSafe(work)) {
            let _ = sender.send(value);
        }
    });
    match spawned {
        Ok(_detached) => Task::perform(receiver, |received| {
            received.map_err(|_canceled| AppError::Worker)
        }),
        Err(source) => Task::done(Err(AppError::io("start", name, source))),
    }
}

/// Loader thread body: always ends with exactly one `Done`, even after a panic.
fn run_load(launch: &LaunchPaths, force: bool, mut sender: mpsc::Sender<LoadEvent>) {
    let outcome = catch_unwind(AssertUnwindSafe(|| load_data(launch, force, &mut sender)));
    let result = match outcome {
        Ok(Ok(data)) => Ok(Handoff::new(data)),
        Ok(Err(error)) => Err(Arc::new(error)),
        Err(_panic) => Err(Arc::new(AppError::Worker)),
    };
    deliver(&mut sender, LoadEvent::Done(result));
}

/// Decodes or imports the bundles and prepares the engine, reporting stages on the way.
fn load_data(
    launch: &LaunchPaths,
    force: bool,
    sender: &mut mpsc::Sender<LoadEvent>,
) -> Result<LoadedData, AppError> {
    let mut report = |stage: Text, fraction: f32| {
        // A full buffer drops this update and a closed one means nobody is listening; the next
        // update or the final result carries the state.
        let _ = sender.try_send(LoadEvent::Progress { stage, fraction });
    };
    let (dataset, index) = load_or_import(launch, force, &mut report)?;
    let stage = if index.is_some() {
        Text::LoadPreparing
    } else {
        Text::LoadExternal
    };
    report(stage, PREPARE_FRACTION);
    prepare(dataset, index)
}

/// Queues an event that must not be lost, waiting for the interface to drain the buffer.
/// Returns early when the receiver is gone, for example after the window closed.
fn deliver(sender: &mut mpsc::Sender<LoadEvent>, mut event: LoadEvent) {
    loop {
        match sender.try_send(event) {
            Ok(()) => return,
            Err(error) if error.is_disconnected() => return,
            Err(error) => {
                event = error.into_inner();
                thread::sleep(RESULT_RETRY);
            }
        }
    }
}

/// Wraps a loader event in the message for its generation.
fn event_message(generation: u64, event: LoadEvent) -> Message {
    match event {
        LoadEvent::Progress { stage, fraction } => Message::Load(LoadMessage::Progress {
            generation,
            stage,
            fraction,
        }),
        LoadEvent::Done(result) => Message::Load(LoadMessage::Finished { generation, result }),
    }
}

#[cfg(test)]
mod tests {
    use iced::futures::channel::mpsc;

    use super::{LoadEvent, deliver, event_message, validate_embedded};
    use crate::AppError;
    use crate::app::i18n::Text;
    use crate::app::message::{LoadMessage, Message};
    use crate::filter::test_fixture::fixture;
    use crate::lang_pack::GameLanguage;

    #[test]
    fn english_has_no_embedded_pack() {
        let result = validate_embedded(GameLanguage::English, &fixture());
        assert!(matches!(result, Err(AppError::InvalidData(_))));
    }

    #[test]
    fn events_carry_their_generation() {
        let progress = event_message(
            3,
            LoadEvent::Progress {
                stage: Text::LoadEmbedded,
                fraction: 0.5,
            },
        );
        assert!(matches!(
            progress,
            Message::Load(LoadMessage::Progress { generation: 3, .. })
        ));
        let done = event_message(4, LoadEvent::Done(Err(std::sync::Arc::new(AppError::Worker))));
        assert!(matches!(
            done,
            Message::Load(LoadMessage::Finished { generation: 4, .. })
        ));
    }

    #[test]
    fn the_result_waits_for_room_instead_of_being_dropped() {
        let (mut sender, mut receiver) = mpsc::channel(0);
        // One slot per sender: this fills the buffer, so the result below must wait.
        assert!(
            sender
                .try_send(LoadEvent::Progress {
                    stage: Text::LoadStarting,
                    fraction: 0.0,
                })
                .is_ok()
        );
        let delivery = std::thread::spawn(move || {
            deliver(&mut sender, LoadEvent::Done(Err(std::sync::Arc::new(AppError::Worker))));
        });
        let mut seen_done = false;
        for _ in 0..2_000 {
            match receiver.try_next() {
                Ok(Some(LoadEvent::Done(_))) => {
                    seen_done = true;
                    break;
                }
                Ok(Some(LoadEvent::Progress { .. })) => {}
                Ok(None) => break,
                Err(_empty) => std::thread::sleep(std::time::Duration::from_millis(1)),
            }
        }
        assert!(seen_done);
        assert!(delivery.join().is_ok());
    }

    #[test]
    fn delivery_returns_when_nobody_listens() {
        let (mut sender, receiver) = mpsc::channel(0);
        drop(receiver);
        deliver(&mut sender, LoadEvent::Done(Err(std::sync::Arc::new(AppError::Worker))));
    }
}
