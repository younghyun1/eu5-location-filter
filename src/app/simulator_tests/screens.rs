//! Renders the real application over the committed bundles into PNG files for visual review.
//!
//! Ignored by default: it needs `EU5_SCREENSHOT_DIR` and writes files. Run it with
//! `EU5_SCREENSHOT_DIR=<dir> cargo test --lib screens -- --ignored`.

use std::borrow::Cow;
use std::path::PathBuf;

use iced::{Settings, Size};
use iced_test::Simulator;

use super::harness::{Harness, ui_error};
use crate::AppError;
use crate::app::context::Platform;
use crate::app::i18n::{FontPack, Locale};
use crate::app::load::{self, prepare};
use crate::app::message::{GridMessage, Message, SelectionSource, SettingsMessage};
use crate::app::state::App;
use crate::app::theme::{Mode, PALETTE_DARK, PALETTE_LIGHT, Palette};
use crate::app::view::{theme, view};

/// Every embedded face and script pack, so each locale draws with its real font.
fn settings() -> Settings {
    let mut fonts: Vec<Cow<'static, [u8]>> = load::embedded_faces().map(Cow::Borrowed).collect();
    for pack in FontPack::ALL {
        if let Some(bytes) = load::embedded_pack(pack) {
            fonts.push(Cow::Borrowed(bytes));
        }
    }
    Settings {
        fonts,
        ..Settings::default()
    }
}

fn real_app(locale: Locale, palette: Palette, size: Size) -> Result<Harness, AppError> {
    let (dataset, index) = crate::embedded::load()?;
    let data = prepare(dataset, Some(index))?;
    let app = App::with_data(Platform::Desktop, locale, palette, data);
    Ok(Harness::from_app(app, size))
}

fn save(harness: &Harness, directory: &std::path::Path, name: &str) -> Result<(), AppError> {
    let mut ui: Simulator<'_, Message> =
        Simulator::with_size(settings(), harness.app.viewport, view(&harness.app));
    let snapshot = ui.snapshot(&theme(&harness.app)).map_err(ui_error)?;
    let created = snapshot
        .matches_image(directory.join(name))
        .map_err(ui_error)?;
    assert!(
        created,
        "{name} differs from an existing file; use an empty directory"
    );
    Ok(())
}

#[test]
#[ignore = "writes PNG files into EU5_SCREENSHOT_DIR for visual review"]
fn renders_review_screens() -> Result<(), AppError> {
    let Some(directory) = std::env::var_os("EU5_SCREENSHOT_DIR").map(PathBuf::from) else {
        return Err(AppError::InvalidData(
            "set EU5_SCREENSHOT_DIR to an empty directory".to_owned(),
        ));
    };
    let wide = Size::new(1440.0, 900.0);

    let mut dark = real_app(Locale::EnUs, PALETTE_DARK, wide)?;
    save(&dark, &directory, "wide-dark-en")?;
    if let Some(id) = dark.results()?.get(3).copied() {
        dark.send(Message::Grid(GridMessage::Select {
            id,
            source: SelectionSource::Pointer,
        }));
    }
    save(&dark, &directory, "wide-dark-en-selected")?;

    let mut light = real_app(Locale::EnUs, PALETTE_LIGHT, wide)?;
    light.send(Message::Settings(SettingsMessage::Mode(Mode::Light)));
    light.settle();
    save(&light, &directory, "wide-light-en")?;

    let korean = real_app(Locale::KoKr, PALETTE_DARK, wide)?;
    save(&korean, &directory, "wide-dark-ko")?;

    let arabic = real_app(Locale::Ar, PALETTE_LIGHT, wide)?;
    save(&arabic, &directory, "wide-light-ar-rtl")?;

    let narrow = real_app(Locale::EnUs, PALETTE_DARK, Size::new(420.0, 860.0))?;
    save(&narrow, &directory, "narrow-dark-en")?;
    Ok(())
}
