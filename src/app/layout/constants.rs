//! Logical-pixel metrics of the interface. Values match the interface layout document; change
//! both together.

/// Desktop window width at first start.
pub(crate) const DESKTOP_DEFAULT_WIDTH: f32 = 1440.0;
/// Desktop window height at first start.
pub(crate) const DESKTOP_DEFAULT_HEIGHT: f32 = 900.0;
/// Desktop minimum window width; the browser build sets no minimum.
#[cfg(feature = "desktop")]
pub(crate) const DESKTOP_MIN_WIDTH: f32 = 1050.0;
/// Desktop minimum window height.
#[cfg(feature = "desktop")]
pub(crate) const DESKTOP_MIN_HEIGHT: f32 = 680.0;
/// Viewport widths below this use the narrow layout; matches the site's mobile limit of 767 px.
pub(crate) const NARROW_BREAKPOINT: f32 = 768.0;
/// Smallest grid width the wide layout keeps beside docked panes.
pub(crate) const GRID_MIN_WIDTH: f32 = 420.0;
/// Wide header height.
pub(crate) const HEADER_HEIGHT: f32 = 56.0;
/// Wide header padding at both inline edges.
pub(crate) const HEADER_PADDING_INLINE: f32 = 16.0;
/// Gap between header items.
pub(crate) const HEADER_GAP: f32 = 12.0;
/// Brand slot width when the rail is collapsed or the header lacks room.
pub(crate) const HEADER_BRAND_MIN_WIDTH: f32 = 184.0;
/// Smallest global search field width.
pub(crate) const SEARCH_MIN_WIDTH: f32 = 240.0;
/// Largest global search field width.
pub(crate) const SEARCH_MAX_WIDTH: f32 = 560.0;
/// Wide global search field height.
pub(crate) const SEARCH_HEIGHT: f32 = 36.0;
/// Fixed slot for result count and build text.
pub(crate) const HEADER_STATUS_WIDTH: f32 = 200.0;
/// Fixed width of Language, Appearance and About buttons; equal to BUTTON_WIDTH.
pub(crate) const HEADER_BUTTON_WIDTH: f32 = 104.0;
/// Height of each of the two control rows of the narrow header.
pub(crate) const NARROW_HEADER_ROW_HEIGHT: f32 = 52.0;
/// Height of the narrow header status line.
pub(crate) const NARROW_STATUS_HEIGHT: f32 = 24.0;
/// Total narrow header height: two control rows plus the status line.
pub(crate) const NARROW_HEADER_HEIGHT: f32 = 128.0;
/// Narrow header and toolbar padding at both inline edges.
pub(crate) const NARROW_PADDING_INLINE: f32 = 12.0;
/// Gap between the equal-width button cells of the narrow header's first row.
pub(crate) const NARROW_HEADER_GAP: f32 = 8.0;
/// Fixed width of text buttons: Columns, Show details, Retry, Rebuild data, Close, link buttons; labels clip with a tooltip.
pub(crate) const BUTTON_WIDTH: f32 = 104.0;
/// Fixed width of Clear all in the rail header and Dismiss in the notice bar.
pub(crate) const BUTTON_WIDTH_COMPACT: f32 = 80.0;
/// Height of text buttons in the wide layout.
pub(crate) const BUTTON_HEIGHT: f32 = 32.0;
/// Inline padding inside text buttons.
pub(crate) const BUTTON_PADDING_INLINE: f32 = 12.0;
/// Square icon button size in the wide layout.
pub(crate) const ICON_BUTTON_SIZE: f32 = 28.0;
/// Minimum button, field and icon target in the narrow layout.
pub(crate) const TOUCH_TARGET: f32 = 44.0;
/// Height of rail text fields in the wide layout.
pub(crate) const INPUT_HEIGHT: f32 = 32.0;
/// Inline padding inside text fields.
pub(crate) const INPUT_PADDING_INLINE: f32 = 8.0;
/// Checkbox square size.
pub(crate) const CHECKBOX_SIZE: f32 = 16.0;
/// Gap between a checkbox and its label.
pub(crate) const CHECK_LABEL_GAP: f32 = 8.0;
/// Hairline border width.
pub(crate) const BORDER_WIDTH: f32 = 1.0;
/// Keyboard focus ring width.
pub(crate) const FOCUS_RING_WIDTH: f32 = 2.0;
/// Corner radius of panels, buttons, fields, menus and swatches.
pub(crate) const RADIUS: f32 = 2.0;
/// Corner radius of count badges.
pub(crate) const RADIUS_PILL: f32 = 999.0;
/// Filter rail width at first start.
pub(crate) const RAIL_DEFAULT_WIDTH: f32 = 280.0;
/// Filter rail minimum width.
pub(crate) const RAIL_MIN_WIDTH: f32 = 240.0;
/// Filter rail maximum width.
pub(crate) const RAIL_MAX_WIDTH: f32 = 420.0;
/// Width of the collapsed rail strip holding the expand button and active-filter badge.
pub(crate) const RAIL_COLLAPSED_WIDTH: f32 = 40.0;
/// Pinned rail header height.
pub(crate) const RAIL_HEADER_HEIGHT: f32 = 44.0;
/// Rail content padding at both inline edges.
pub(crate) const RAIL_PADDING_INLINE: f32 = 12.0;
/// Rail content padding at top and bottom.
pub(crate) const RAIL_PADDING_BLOCK: f32 = 12.0;
/// Space above each rail section title.
pub(crate) const RAIL_SECTION_GAP: f32 = 16.0;
/// Space between facet groups.
pub(crate) const RAIL_GROUP_GAP: f32 = 8.0;
/// Rail section title row height.
pub(crate) const SECTION_TITLE_HEIGHT: f32 = 24.0;
/// Lists with more options than this show a search field; 0 keeps search on every list.
pub(crate) const CHECKLIST_SEARCH_THRESHOLD: usize = 0;
/// Checklist option row height in the wide layout.
pub(crate) const CHECKLIST_ROW_HEIGHT: f32 = 28.0;
/// Checklist option row height in the narrow layout.
pub(crate) const CHECKLIST_ROW_HEIGHT_TOUCH: f32 = 40.0;
/// Rows shown before a checklist scrolls.
pub(crate) const CHECKLIST_MAX_VISIBLE_ROWS: usize = 7;
/// Inline padding inside checklist rows.
pub(crate) const CHECKLIST_PADDING_INLINE: f32 = 8.0;
/// Accent outline inside the active checklist option while the list has keyboard focus.
pub(crate) const ACTIVE_OPTION_OUTLINE: f32 = 1.0;
/// Gap between the minimum and maximum fields of a range.
pub(crate) const RANGE_GAP: f32 = 8.0;
/// Pointer target width of a pane splitter.
pub(crate) const SPLITTER_HIT_WIDTH: f32 = 8.0;
/// Visible splitter line at rest.
pub(crate) const SPLITTER_LINE_WIDTH: f32 = 1.0;
/// Visible splitter line while hovered, focused or dragged.
pub(crate) const SPLITTER_ACTIVE_LINE_WIDTH: f32 = 2.0;
/// Pane width change per arrow key on a focused splitter.
pub(crate) const PANE_KEY_STEP: f32 = 16.0;
/// Grid toolbar height in the wide layout.
pub(crate) const GRID_TOOLBAR_HEIGHT: f32 = 40.0;
/// Grid toolbar height in the narrow layout.
pub(crate) const NARROW_TOOLBAR_HEIGHT: f32 = 52.0;
/// Colour square before each legend label.
pub(crate) const LEGEND_MARK_SIZE: f32 = 8.0;
/// Grid column header row height.
pub(crate) const GRID_HEADER_HEIGHT: f32 = 36.0;
/// Grid body row height in the wide layout.
pub(crate) const GRID_ROW_HEIGHT: f32 = 32.0;
/// Grid body row height in the narrow layout.
pub(crate) const GRID_ROW_HEIGHT_TOUCH: f32 = 40.0;
/// Space before the first column; holds the selected-row bar and keeps the ported width test at 1346.
pub(crate) const GRID_INLINE_INSET: f32 = 10.0;
/// Inline padding inside every cell.
pub(crate) const GRID_CELL_PADDING_INLINE: f32 = 8.0;
/// Accent bar at the inline start of the selected row.
pub(crate) const SELECTED_BAR_WIDTH: f32 = 3.0;
/// Accent underline under the active column's header while the grid has keyboard focus.
pub(crate) const ACTIVE_COLUMN_UNDERLINE: f32 = 2.0;
/// Narrowest column.
pub(crate) const COLUMN_MIN_WIDTH: f32 = 54.0;
/// Widest column.
pub(crate) const COLUMN_MAX_WIDTH: f32 = 460.0;
/// Pointer target centred on each column's inline-end edge.
pub(crate) const COLUMN_RESIZE_HIT_WIDTH: f32 = 8.0;
/// Column width change per Shift+arrow.
pub(crate) const COLUMN_KEY_STEP: f32 = 16.0;
/// Space reserved at a header cell's inline end for the sort arrow.
pub(crate) const SORT_GLYPH_WIDTH: f32 = 14.0;
/// Colour swatch square in the colour column.
pub(crate) const SWATCH_SIZE: f32 = 18.0;
/// Gap between swatch and hex text.
pub(crate) const SWATCH_GAP: f32 = 8.0;
/// Scrollbar lane width in grid and checklists.
pub(crate) const SCROLLBAR_WIDTH: f32 = 10.0;
/// Drawn thumb width with a pointer.
pub(crate) const SCROLLBAR_THUMB_WIDTH: f32 = 6.0;
/// Drawn thumb width in the narrow layout.
pub(crate) const SCROLLBAR_THUMB_WIDTH_TOUCH: f32 = 4.0;
/// Shortest thumb length.
pub(crate) const SCROLLBAR_MIN_THUMB: f32 = 24.0;
/// Pixels per wheel line, the same factor iced's scrollable uses.
pub(crate) const WHEEL_LINE_PIXELS: f32 = 60.0;
/// Touch movement below this is a tap; above it the drag locks to its dominant axis.
pub(crate) const TOUCH_SLOP: f32 = 8.0;
/// Detail pane width at first start.
pub(crate) const DETAIL_DEFAULT_WIDTH: f32 = 340.0;
/// Detail pane minimum width.
pub(crate) const DETAIL_MIN_WIDTH: f32 = 280.0;
/// Detail pane maximum width.
pub(crate) const DETAIL_MAX_WIDTH: f32 = 520.0;
/// Pinned detail header height.
pub(crate) const DETAIL_HEADER_HEIGHT: f32 = 44.0;
/// Detail content padding on all sides.
pub(crate) const DETAIL_PADDING: f32 = 16.0;
/// Height of the full-width colour block.
pub(crate) const DETAIL_SWATCH_HEIGHT: f32 = 72.0;
/// Label column of the two-column field list.
pub(crate) const DETAIL_LABEL_WIDTH: f32 = 128.0;
/// Minimum height of one field row; values wrap below it.
pub(crate) const DETAIL_FIELD_MIN_HEIGHT: f32 = 28.0;
/// Space above each field group title.
pub(crate) const DETAIL_GROUP_GAP: f32 = 16.0;
/// Width of column, language and appearance menus.
pub(crate) const MENU_WIDTH: f32 = 260.0;
/// Tallest menu before its list scrolls.
pub(crate) const MENU_MAX_HEIGHT: f32 = 480.0;
/// Padding inside menus.
pub(crate) const MENU_PADDING: f32 = 8.0;
/// Gap between a menu and its opener.
pub(crate) const MENU_OFFSET: f32 = 4.0;
/// About dialog width, limited to the viewport minus 2 * DIALOG_MARGIN.
pub(crate) const DIALOG_WIDTH: f32 = 560.0;
/// About dialog maximum height; content scrolls inside.
pub(crate) const DIALOG_MAX_HEIGHT: f32 = 640.0;
/// Smallest distance between a dialog and the viewport edge.
pub(crate) const DIALOG_MARGIN: f32 = 16.0;
/// About dialog padding.
pub(crate) const DIALOG_PADDING: f32 = 24.0;
/// Vertical gap between About dialog blocks.
pub(crate) const DIALOG_GAP: f32 = 12.0;
/// Loading card width, limited to the viewport minus 2 * DIALOG_MARGIN.
pub(crate) const LOADING_CARD_WIDTH: f32 = 440.0;
/// Load-error card width, limited the same way.
pub(crate) const ERROR_CARD_WIDTH: f32 = 640.0;
/// Progress bar height.
pub(crate) const PROGRESS_HEIGHT: f32 = 4.0;
/// Height of the notice bar under the header.
pub(crate) const NOTICE_HEIGHT: f32 = 40.0;
/// Narrow filter drawer width, limited to the viewport minus DRAWER_EDGE_GAP.
pub(crate) const DRAWER_RAIL_WIDTH: f32 = 320.0;
/// Narrow detail drawer width, limited the same way.
pub(crate) const DRAWER_DETAIL_WIDTH: f32 = 360.0;
/// Scrim strip that stays visible beside an open drawer.
pub(crate) const DRAWER_EDGE_GAP: f32 = 48.0;
