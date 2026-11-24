//! Unit tests for the root view's pure decisions: surface stacking, menu anchors and the
//! direction-aware popover padding.

use iced::Size;

use super::plan::{self, Menu, SurfacePlan, TOOLBAR_GAP, TOOLBAR_PADDING_INLINE};
use super::surfaces::end_padding;
use crate::app::layout::{
    BUTTON_HEIGHT, BUTTON_WIDTH, DIALOG_MARGIN, Direction, Drawer, GRID_TOOLBAR_HEIGHT,
    HEADER_BRAND_MIN_WIDTH, HEADER_BUTTON_WIDTH, HEADER_GAP, HEADER_HEIGHT,
    HEADER_PADDING_INLINE, LayoutMode, MENU_MAX_HEIGHT, MENU_OFFSET, MENU_WIDTH,
    NARROW_HEADER_HEIGHT, NARROW_TOOLBAR_HEIGHT, PaneLayout, PaneSlot, SPLITTER_HIT_WIDTH,
};
use crate::app::state::{Notice, Surface};

fn panes(rail: PaneSlot, detail: PaneSlot) -> PaneLayout {
    PaneLayout {
        rail,
        detail,
        grid_width: 600.0,
        brand_width: HEADER_BRAND_MIN_WIDTH,
        detail_forced_hidden: false,
    }
}

fn docked() -> PaneLayout {
    panes(PaneSlot::Docked(280.0), PaneSlot::Docked(340.0))
}

const DESKTOP: Size = Size::new(1440.0, 900.0);
const PHONE: Size = Size::new(360.0, 640.0);

#[test]
fn nothing_open_plans_no_surface() {
    let planned = plan::plan(None, None, &docked(), LayoutMode::Wide);
    assert_eq!(
        planned,
        SurfacePlan {
            menu: None,
            notice: None,
            drawer: None,
            about: false,
        }
    );
}

#[test]
fn surfaces_map_to_menus_or_the_about_dialog() {
    let cases = [
        (Surface::ColumnMenu, Some(Menu::Columns), false),
        (Surface::LanguageMenu, Some(Menu::Language), false),
        (Surface::AppearanceMenu, Some(Menu::Appearance), false),
        (Surface::About, None, true),
    ];
    for (surface, menu, about) in cases {
        let planned = plan::plan(
            Some(surface),
            Some(Notice::FontPack),
            &docked(),
            LayoutMode::Wide,
        );
        assert_eq!(planned.menu, menu);
        assert_eq!(planned.about, about);
        assert_eq!(planned.notice, Some(Notice::FontPack));
    }
}

#[test]
fn drawers_appear_only_in_the_narrow_layout() {
    let filters = panes(PaneSlot::Drawer(312.0), PaneSlot::Hidden);
    let details = panes(PaneSlot::Hidden, PaneSlot::Drawer(312.0));
    let closed = panes(PaneSlot::Hidden, PaneSlot::Hidden);
    assert_eq!(
        plan::plan(None, None, &filters, LayoutMode::Narrow).drawer,
        Some((Drawer::Filters, 312.0))
    );
    assert_eq!(
        plan::plan(None, None, &details, LayoutMode::Narrow).drawer,
        Some((Drawer::Details, 312.0))
    );
    assert_eq!(plan::plan(None, None, &closed, LayoutMode::Narrow).drawer, None);
    assert_eq!(plan::plan(None, None, &filters, LayoutMode::Wide).drawer, None);
}

#[test]
fn header_menus_line_up_with_their_buttons() {
    let button_bottom = HEADER_HEIGHT - (HEADER_HEIGHT - BUTTON_HEIGHT) / 2.0;
    let language = plan::menu_anchor(Menu::Language, LayoutMode::Wide, &docked(), DESKTOP);
    let appearance = plan::menu_anchor(Menu::Appearance, LayoutMode::Wide, &docked(), DESKTOP);
    assert_eq!(language.top, button_bottom + MENU_OFFSET);
    assert_eq!(appearance.top, language.top);
    assert_eq!(
        appearance.end_inset,
        HEADER_PADDING_INLINE + HEADER_BUTTON_WIDTH + HEADER_GAP
    );
    assert_eq!(
        language.end_inset,
        appearance.end_inset + HEADER_BUTTON_WIDTH + HEADER_GAP
    );
    assert_eq!(language.width, MENU_WIDTH);
    assert_eq!(language.max_height, MENU_MAX_HEIGHT);
}

#[test]
fn column_menu_follows_the_detail_pane() {
    let top = HEADER_HEIGHT + GRID_TOOLBAR_HEIGHT + MENU_OFFSET;
    let with_detail = plan::menu_anchor(Menu::Columns, LayoutMode::Wide, &docked(), DESKTOP);
    assert_eq!(with_detail.top, top);
    assert_eq!(
        with_detail.end_inset,
        340.0 + SPLITTER_HIT_WIDTH + TOOLBAR_PADDING_INLINE
    );
    let hidden = panes(PaneSlot::Docked(280.0), PaneSlot::Hidden);
    let without_detail = plan::menu_anchor(Menu::Columns, LayoutMode::Wide, &hidden, DESKTOP);
    assert_eq!(
        without_detail.end_inset,
        TOOLBAR_PADDING_INLINE + BUTTON_WIDTH + TOOLBAR_GAP
    );
}

#[test]
fn narrow_menus_keep_the_dialog_margin() {
    let closed = panes(PaneSlot::Hidden, PaneSlot::Hidden);
    let anchor = plan::menu_anchor(Menu::Columns, LayoutMode::Narrow, &closed, PHONE);
    assert_eq!(anchor.top, NARROW_HEADER_HEIGHT + NARROW_TOOLBAR_HEIGHT + MENU_OFFSET);
    assert_eq!(anchor.end_inset, DIALOG_MARGIN);
    assert_eq!(anchor.width, MENU_WIDTH);
    assert_eq!(anchor.max_height, PHONE.height - anchor.top - DIALOG_MARGIN);
    let tiny = plan::menu_anchor(Menu::Columns, LayoutMode::Narrow, &closed, Size::new(200.0, 150.0));
    assert_eq!(tiny.width, 200.0 - 2.0 * DIALOG_MARGIN);
    assert_eq!(tiny.max_height, 0.0);
}

#[test]
fn notice_sits_under_the_header_of_each_mode() {
    assert_eq!(plan::header_height(LayoutMode::Wide), HEADER_HEIGHT);
    assert_eq!(plan::header_height(LayoutMode::Narrow), NARROW_HEADER_HEIGHT);
}

#[test]
fn popover_padding_mirrors_for_right_to_left() {
    let ltr = end_padding(Direction::Ltr, 48.0, 132.0);
    assert_eq!((ltr.top, ltr.right, ltr.bottom, ltr.left), (48.0, 132.0, 0.0, 0.0));
    let rtl = end_padding(Direction::Rtl, 48.0, 132.0);
    assert_eq!((rtl.top, rtl.right, rtl.bottom, rtl.left), (48.0, 0.0, 0.0, 132.0));
}
