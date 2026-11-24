//! Unit tests of the About dialog.

use super::{data_line, version_line, view};
use crate::app::context::Platform;
use crate::app::i18n::Locale;
use crate::app::layout::LayoutMode;
use crate::app::panes::overlays::kit::test_support::{context, contexts};
use crate::model::EU5_GAME_VERSION;

#[test]
fn the_version_line_names_this_crate_version() {
    for ctx in contexts() {
        let line = version_line(ctx);
        assert!(line.contains(env!("CARGO_PKG_VERSION")), "{line}");
        assert!(!line.contains('{'), "{line}");
    }
}

#[test]
fn the_data_line_names_game_version_and_build() {
    let ctx = context(Platform::Desktop, LayoutMode::Wide, Locale::EnUs);
    let line = data_line(ctx, Some(26_547_371));
    assert!(line.contains(EU5_GAME_VERSION), "{line}");
    assert!(line.contains("26547371"), "build is not grouped: {line}");
    let pending = data_line(ctx, None);
    assert_eq!(pending, ctx.t(crate::app::i18n::Text::HeaderBuildPending));
}

#[test]
fn the_dialog_builds_in_every_state() {
    for ctx in contexts() {
        for expanded in [false, true] {
            for build in [None, Some(1_u64)] {
                let _dialog = view(ctx, build, expanded);
            }
        }
    }
}
