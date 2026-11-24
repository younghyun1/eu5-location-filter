//! Tests of the header status text and of building both layouts for both platforms.

use iced::Element;

use super::{HeaderStatus, status, view};
use crate::app::context::{Platform, ViewContext};
use crate::app::i18n::Locale;
use crate::app::layout::{Direction, LayoutMode};
use crate::app::message::Message;
use crate::app::state::Surface;
use crate::app::theme::PALETTE_DARK;

fn context(direction: Direction, mode: LayoutMode, platform: Platform) -> ViewContext {
    ViewContext {
        palette: PALETTE_DARK,
        locale: Locale::EnUs,
        direction,
        mode,
        platform,
    }
}

#[test]
fn status_before_data_shows_only_the_loading_text() {
    let ctx = context(Direction::Ltr, LayoutMode::Wide, Platform::Desktop);
    let status = HeaderStatus::default();
    assert_eq!(status::count_text(ctx, status), "");
    assert_eq!(status::build_text(ctx, status), "Loading data");
}

#[test]
fn status_after_data_shows_count_and_build() {
    let ctx = context(Direction::Ltr, LayoutMode::Wide, Platform::Desktop);
    let status = HeaderStatus {
        result_count: Some(1),
        build_id: Some(26_547_371),
        active_filters: 0,
    };
    assert_eq!(status::count_text(ctx, status), "1 location");
    assert_eq!(
        status::build_text(ctx, status),
        "EU5 1.4.8 | Build 26547371"
    );
    let many = HeaderStatus {
        result_count: Some(12_345),
        ..status
    };
    assert_eq!(status::count_text(ctx, many), "12,345 locations");
}

#[test]
fn header_builds_for_every_layout_platform_and_direction() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        for mode in [LayoutMode::Wide, LayoutMode::Narrow] {
            for platform in [Platform::Desktop, Platform::Browser] {
                for open in [None, Some(Surface::LanguageMenu), Some(Surface::About)] {
                    let ctx = context(direction, mode, platform);
                    let status = HeaderStatus {
                        result_count: Some(7),
                        build_id: Some(1),
                        active_filters: 3,
                    };
                    let _: Element<'_, Message> = view(ctx, "query", status, 280.0, open);
                }
            }
        }
    }
}
