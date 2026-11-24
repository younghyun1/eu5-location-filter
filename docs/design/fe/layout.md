# Interface layout

The interface is written with iced 0.14 and follows the cyhdev site's design language: warm paper and ink neutrals, one accent family, IBM Plex Sans for reading text and IBM Plex Mono for technical text, hairline borders, and shadows only on floating surfaces. It is a free design rather than an imitation of a desktop toolkit. Values below are logical pixels; the named constants live in `src/app/layout.rs` and `src/app/theme.rs`, and this document gives each value next to its name so the two stay comparable.

## Principles

- Every function of the previous interface stays: global search, result count and build text; a collapsible, resizable filter rail; the virtualized, sortable, resizable result grid with a column menu; the collapsible, resizable detail pane; loading progress, load errors with desktop retry and rebuild, and the About dialog.
- A control's size never comes from its label. Buttons, column headers and menu items have fixed sizes; a label that does not fit is clipped at the control edge and the full text is shown as a tooltip. Changing the language must not move any control. iced 0.14 has no text ellipsis, so text is clipped rather than elided.
- The grid and every checklist draw only the visible rows. Cell text borrows dataset strings, numbers format into one reused buffer, and no per-row allocation happens outside the visible window. iced's text cache drops entries unused since the previous frame, so drawing only visible cells keeps it bounded.
- Colours come from 12 host tokens and the mode. Nothing in the interface uses a literal colour except black scrims and shadows, which the site also allows.
- No screen-reader tree exists in iced 0.14. Keyboard operation is complete instead, and every control has visible text or a tooltip.

## Regions in the wide layout

The window is a column: header, then a body row. The body row holds, in inline order, the filter rail, a splitter, the results region, a splitter and the detail pane. In a left-to-right language the rail is on the left; in a right-to-left language the whole row is mirrored.

### Header

Height `HEADER_HEIGHT` (56), ground `surface`, bottom border `line`, inline padding `HEADER_PADDING_INLINE` (16), item gap `HEADER_GAP` (12). Items in inline order:

- Brand `app.title` in Mono Medium `TEXT_BRAND` (15), `ink`. While the rail is docked and expanded its slot is the larger of `HEADER_BRAND_MIN_WIDTH` (184) and the rail width plus `SPLITTER_HIT_WIDTH` (8) minus `HEADER_PADDING_INLINE` (16) and `HEADER_GAP` (12), so the search field starts where the grid starts; otherwise the slot is `HEADER_BRAND_MIN_WIDTH`.
- Global search, a text field `SEARCH_HEIGHT` (36) high, between `SEARCH_MIN_WIDTH` (240) and `SEARCH_MAX_WIDTH` (560) wide, placeholder `header.search.placeholder`. It filters as the user types.
- Status slot, fixed at `HEADER_STATUS_WIDTH` (200): the result count (`header.result_count`) in Mono `TEXT_META` (12) `ink` on the first line and the build text (`header.build`) in Mono `TEXT_META` (12) `ink_faint` on the second.
- Desktop only: Language and Appearance buttons, each `HEADER_BUTTON_WIDTH` (104) wide and `BUTTON_HEIGHT` (32) high, each opening a menu below itself. The browser build hides both, because the site sends language, direction, palette and motion.
- About button, `HEADER_BUTTON_WIDTH` (104) wide.

When the header lacks room, items yield in a fixed order: first the brand slot gives up the rail alignment and drops to `HEADER_BRAND_MIN_WIDTH`, then the search field shrinks to `SEARCH_MIN_WIDTH`. At those minimums the desktop header needs 1028 px and the browser header, without Language and Appearance, needs 796 px, so both fit at their smallest wide widths (`DESKTOP_MIN_WIDTH` and `NARROW_BREAKPOINT`).

### Filter rail

Default width `RAIL_DEFAULT_WIDTH` (280), limits `RAIL_MIN_WIDTH` (240) to `RAIL_MAX_WIDTH` (420), ground `surface`, inline-end border `line`. A pinned header `RAIL_HEADER_HEIGHT` (44) high holds the title `rail.title` in Sans SemiBold `TEXT_BODY` (13), a pill badge with the number of active filter fields (`rail.active_count` as its tooltip), the Clear all button (`BUTTON_WIDTH_COMPACT`, 80) and the collapse button. Collapsing leaves a strip `RAIL_COLLAPSED_WIDTH` (40) wide with the expand button and the badge; the stored width is kept.

The scrollable content has padding `RAIL_PADDING_INLINE` (12) inline and `RAIL_PADDING_BLOCK` (12) block. Sections are Location, Geography, Terrain, Society, Economy and Map data; each title is Mono Medium `TEXT_CAPTION` (11) in `ink_faint` on a row `SECTION_TITLE_HEIGHT` (24) high with `RAIL_SECTION_GAP` (16) above it. Facet groups are separated by `RAIL_GROUP_GAP` (8).

A facet group header is `FACET_HEADER_HEIGHT` (32) high: a fold button with a chevron, the facet title in Sans Medium `TEXT_SMALL` (12) `ink`, a count badge when options are selected, and a clear button (multiplication sign) that appears only with a selection. Type starts expanded and every other group starts folded, so the rail opens short; a folded group keeps its badge. An expanded group shows a search field `INPUT_HEIGHT` (32) high when it has more than `CHECKLIST_SEARCH_THRESHOLD` (0) options, which is every list, then the checklist: rows `CHECKLIST_ROW_HEIGHT` (28) high, at most `CHECKLIST_MAX_VISIBLE_ROWS` (7) visible before it scrolls, checkbox `CHECKBOX_SIZE` (16) at the inline start with `CHECK_LABEL_GAP` (8) before the label, inline padding `CHECKLIST_PADDING_INLINE` (8). The list has a `line` border and `RADIUS` (2) corners. Selections inside one group combine with OR, groups combine with AND, and an empty group means any value. An empty search result shows `checklist.empty`.

Toggles (Show impassable, Food-producing only) are checkbox rows of the same metrics. The exact-colour field and the range fields are text fields `INPUT_HEIGHT` (32) high; a range row holds a minimum and a maximum field with `RANGE_GAP` (8) between them under a Sans `TEXT_SMALL` (12) `ink_muted` label. An invalid value turns that field's border `danger` and shows its message in Sans `TEXT_SMALL` (12) `danger` directly below the row; the active result set does not change until the input is valid.

### Splitters

Each splitter occupies `SPLITTER_HIT_WIDTH` (8) of layout width between its neighbours as its pointer target, centred on a visible line `SPLITTER_LINE_WIDTH` (1) wide in `line`. Hovered, focused or dragged, the line becomes `SPLITTER_ACTIVE_LINE_WIDTH` (2) wide in `accent` and the cursor becomes `mouse::Interaction::ResizingColumn`. A focused splitter moves by `PANE_KEY_STEP` (16) per arrow key. The rail splitter is absent while the rail is collapsed, and the detail splitter while the detail pane is hidden.

### Results

The results region fills the remaining width and never goes below `GRID_MIN_WIDTH` (420). Its toolbar is `GRID_TOOLBAR_HEIGHT` (40) high, ground `surface_2`, bottom border `line`. It shows the legend (a `LEGEND_MARK_SIZE` (8) square in `food` with `grid.legend.food`, a square of the same size in `precious` with `grid.legend.precious`, both Mono Medium `TEXT_CAPTION` (11)), then at the inline end the Columns button and, while the detail pane is hidden, the Show details button, both `BUTTON_WIDTH` (104) wide. The legend hides when the toolbar is too narrow for it and the buttons.

The grid is one widget that owns its header row, body and scrollbars.

- Header row `GRID_HEADER_HEIGHT` (36) high, ground `surface_2`, bottom border `line_strong`. Labels are Sans Medium `TEXT_SMALL` (12) `ink_muted`, aligned like their column's cells; the sorted column's label is `ink` and shows an up or down arrow in the `SORT_GLYPH_WIDTH` (14) reserved at its inline end. Clicking a header sorts by it; clicking again reverses. Each column's inline-end edge carries a resize target `COLUMN_RESIZE_HIT_WIDTH` (8) wide, a `line` divider at rest and an `accent` divider while hovered or dragged. Widths are clamped to `COLUMN_MIN_WIDTH` (54) to `COLUMN_MAX_WIDTH` (460).
- Body rows `GRID_ROW_HEIGHT` (32) high. The first column starts after an inset of `GRID_INLINE_INSET` (10), which holds the selected-row bar. Cells have `GRID_CELL_PADDING_INLINE` (8) inline padding and clip at their edges; columns are separated by `line` dividers and rows by nothing but zebra grounds. Text cells use Sans Regular `TEXT_BODY` (13); the name column uses Sans Medium; identifiers, numbers, hex and RGB use Mono `TEXT_META` (12). Numeric columns align to the inline end. The colour column draws a `SWATCH_SIZE` (18) swatch with a `line_strong` border and `RADIUS` corners, then the hex value after `SWATCH_GAP` (8).
- Scrolling: vertical wheel scrolls rows, Shift with the wheel scrolls columns, middle-button drag pans horizontally, and one wheel line is `WHEEL_LINE_PIXELS` (60) px. Scrollbars occupy a lane `SCROLLBAR_WIDTH` (10) wide at the inline end and the block end, reserved inside the grid bounds so the body viewport excludes it, with a thumb `SCROLLBAR_THUMB_WIDTH` (6) wide and at least `SCROLLBAR_MIN_THUMB` (24) long. The vertical scrollbar stays at the visible edge however wide the columns are.
- Selection: a click selects a row and shows it in the detail pane. Filtering keeps the selection when the location is still in the result set and clears it otherwise. When no location matches, the body shows `grid.empty.title` and `grid.empty.body` centred, with the Clear all action in the rail.

### Column menu

The Columns button opens a popover `MENU_WIDTH` (260) wide and at most `MENU_MAX_HEIGHT` (480) high, `MENU_OFFSET` (4) below the toolbar and aligned to the button's inline end, ground `surface`, border `line_strong`, shadow. It has the title `column_menu.title`, a close button, and a checklist of all 26 columns in default order without a search field. Unchecking the last visible column is refused and `column_menu.keep_one` appears as the hint. The same button closes the menu; a press outside or Escape also closes it.

### Detail pane

Default width `DETAIL_DEFAULT_WIDTH` (340), limits `DETAIL_MIN_WIDTH` (280) to `DETAIL_MAX_WIDTH` (520), ground `surface`, inline-start border `line`. A pinned header `DETAIL_HEADER_HEIGHT` (44) high holds `detail.title` in Mono Medium `TEXT_CAPTION` (11) `ink_faint` and the hide button. Hiding returns its width to the grid and keeps the stored width; the grid toolbar's Show details button restores it.

The scrollable content has `DETAIL_PADDING` (16) padding: a colour block `DETAIL_SWATCH_HEIGHT` (72) high across the full width with a `line_strong` border, the hex value and `format.rgb` below it in Mono `TEXT_META` (12) `ink_muted`, the location name in Sans SemiBold `TEXT_HEADING` (20), the symbol key in Mono `TEXT_META` (12) `ink_muted`, the breadcrumb from continent to province in Sans `TEXT_SMALL` (12) `ink_muted` with `ink_faint` separators, and the field groups. Each group title is Mono Medium `TEXT_CAPTION` (11) `ink_faint` with `DETAIL_GROUP_GAP` (16) above it; each field row is at least `DETAIL_FIELD_MIN_HEIGHT` (28) high with the label in a `DETAIL_LABEL_WIDTH` (128) column (Sans `TEXT_SMALL` (12) `ink_muted`) and the wrapping value (Sans `TEXT_BODY` (13) `ink`, line height 1.45). Groups are classification, society, economy, coast and movement, river, and the static population capacity breakdown (total, vegetation capacity, closeness to the equator, static modifier) with a note that it excludes campaign state. Without a selection the pane shows `detail.empty.title` in Sans SemiBold `TEXT_TITLE` (16) and `detail.empty.body` in `ink_muted`, centred.

### Loading, errors and notices

- Loading: a `loading_veil` covers the window, and a card `LOADING_CARD_WIDTH` (440) wide (ground `surface`, border `line_strong`, shadow) shows the stage text in Sans SemiBold `TEXT_TITLE` (16), a progress bar `PROGRESS_HEIGHT` (4) high with a `line` track and an `accent` fill, and `format.progress` in Mono `TEXT_META` (12). The bar reflects reported progress and never animates on its own. The browser paints this shell before decoding the bundle.
- Load error: an `error_veil` covers the window, and a card `ERROR_CARD_WIDTH` (640) wide shows `error.title` in Sans SemiBold `TEXT_HEADING` (20) `danger`, the family message in `ink`, the technical detail in Mono `TEXT_META` (12) `ink_muted` on `surface_2`, and the hint. On desktop it also has the installation directory field and the Retry and Rebuild data buttons; Retry is the primary button. The browser shows `error.hint.web` and no controls.
- Notices: a failed language or font pack, or a pack from another build, shows a bar `NOTICE_HEIGHT` (40) high under the header with the message in `ink`, a `danger` inline-start rule `SELECTED_BAR_WIDTH` (3) wide, and a Dismiss button. The interface keeps running in English names or fallback fonts.

### About dialog

A centred dialog `DIALOG_WIDTH` (560) wide and at most `DIALOG_MAX_HEIGHT` (640) high, at least `DIALOG_MARGIN` (16) from the viewport edges, padding `DIALOG_PADDING` (24), block gap `DIALOG_GAP` (12), ground `surface`, border `line_strong`, shadow, over a `scrim`. It shows the title in Sans SemiBold `TEXT_HEADING` (20), version and data build in Mono `TEXT_META` (12), the author, three link buttons (GitHub, cyhdev.com, LinkedIn), the disclaimer and trademark notice, the licences (source MIT OR Apache-2.0; interface toolkit iced under MIT; IBM Plex and Noto fonts under the SIL Open Font License 1.1) with a button that expands the full licence texts in Mono `TEXT_META` (12) inside the dialog's scroll area, the keyboard table, and Close. There is no other toolkit attribution. A press on the scrim or Escape closes the dialog and returns focus to the About button.

### Desktop menus

Language and Appearance menus use the column menu frame. The language menu lists every supported locale by its own name; selecting one applies it at once and shows `language.game_text_note` when the language has no game text. The Appearance menu has Mode (Light, Dark), Accent (Amber, Lagoon, Cobalt, Orchid, Graphite as swatch buttons) and Animations (On, Off). Choices last for the session.

## Common controls

| Control | Rest | Hover | Pressed | Focused | Disabled |
| --- | --- | --- | --- | --- | --- |
| Button | ground `surface`, border `line_strong`, text `ink` | ground `surface_2` | ground `pressed` | rest plus focus ring | text `ink_faint`, border `line` |
| Primary button | ground `accent`, text `on_accent`, no border | ground `accent_strong` | ground `accent_strong` | rest plus focus ring | as button disabled |
| Icon button | no ground, glyph `ink_muted` | ground `surface_2`, glyph `ink` | ground `pressed` | rest plus focus ring | glyph `ink_faint` |
| Text field | ground `surface`, border `line_strong`, placeholder `ink_faint`, value `ink`, selection `text_selection` | border `ink_faint` | | border 2 px `accent` | value `ink_faint` |
| Invalid text field | border `danger` | border `danger` | | border 2 px `danger` | |
| Checkbox | box border `line_strong`, no fill | border `ink_faint` | | active-option outline | border `line` |
| Checked checkbox | box fill `accent`, check `on_accent` | fill `accent_strong` | | active-option outline | fill `line_strong` |
| Count badge | ground `accent_soft`, text `accent_strong`, Mono Medium 11 px, `RADIUS_PILL` | | | | |

Buttons are `BUTTON_WIDTH` (104) wide, or `BUTTON_WIDTH_COMPACT` (80) for Clear all in the rail header and Dismiss in the notice bar, and `BUTTON_HEIGHT` (32) high with `BUTTON_PADDING_INLINE` (12) inline padding and Sans Medium `TEXT_BODY` (13) labels; icon buttons are `ICON_BUTTON_SIZE` (28) square. All corners use `RADIUS` (2). The focus ring is `FOCUS_RING_WIDTH` (2) in `accent` drawn inside the control's bounds, so focus never changes layout. iced buttons and text fields change style instantly; only the palette mix and drawers animate.

## Typography

Embedded faces: IBM Plex Sans Regular, Medium and SemiBold; IBM Plex Mono Regular and Medium. Scripts outside Latin, Latin Extended, Vietnamese and Cyrillic use lazily loaded font packs; text in those scripts uses the pack's face for both roles, because no monospace face exists for them.

| Role | Face | Size | Weight | Use |
| --- | --- | --- | --- | --- |
| Brand | Mono | 15 | Medium | header brand |
| Heading | Sans | 20 | SemiBold | detail name, About title, error title |
| Title | Sans | 16 | SemiBold | loading stage, empty states |
| Body | Sans | 13 | Regular | cells, options, field values, detail values, About text |
| Body strong | Sans | 13 | Medium | name column, button labels, rail title (SemiBold) |
| Small | Sans | 12 | Regular or Medium | facet titles and column headers (Medium), detail labels, hints, inline errors |
| Meta | Mono | 12 | Regular | result count, build, identifiers, numbers, hex, RGB, technical detail, licence texts |
| Caption | Mono | 11 | Medium | section and group titles, pane captions, legend, badges |

Single-line text uses relative line height `LINE_HEIGHT` (1.3); wrapping text uses `LINE_HEIGHT_PROSE` (1.45). Text fields use `TEXT_BODY` (13) in the wide layout and `TEXT_INPUT_NARROW` (16) in the narrow layout. Headings are written in sentence case in the catalogs and never case-converted. iced has no letter spacing, so captions rely on the Mono face rather than tracking.

## Colour tokens

The palette is the 12 site tokens plus the mode. Derived colours are computed with a per-channel sRGB mix; the ratios are constants in `theme.rs`.

| Token | Surfaces and states |
| --- | --- |
| `paper` | window ground behind panes; zebra mix target; loading and error veils; text on accent grounds |
| `surface` | header, rail, grid body, detail pane, menus, dialogs, cards, text fields, buttons |
| `ink` | primary text, sorted header label, checklist labels, detail values |
| `ink_muted` | secondary text: column headers, detail labels and key, breadcrumb, range labels, icon glyphs |
| `ink_faint` | tertiary text: placeholders, build text, section and pane captions, breadcrumb separators, resting scrollbar thumb, disabled text |
| `line` | hairlines: pane edges, header and toolbar bottoms, column dividers, checklist borders, resting splitter, progress track, `surface_2` mix target |
| `line_strong` | emphasized lines: grid header bottom, button and field borders, menu and dialog borders, swatch borders, pressed mix target |
| `accent` | focus ring, selected-row bar, checked boxes, primary button, progress fill, active splitter and column divider, dragged scrollbar thumb, gold and silver raw materials |
| `accent_strong` | primary button hover, checked box hover, count badge text, river bonus cells |
| `accent_soft` | selected row, text selection, count badge ground, hover mix target |
| `ok` | food-producing raw materials in cells, detail and legend |
| `danger` | invalid field border and message, error title, notice rule |

Derived: `surface_2` = mix(`surface`, `line`, `SURFACE_2_LINE_MIX` (0.5)), which reproduces the site's `--surface-2` in both modes, for toolbars, the grid header, pressed fills and the technical detail block. Odd rows are `zebra` = mix(`surface`, `paper`, `ZEBRA_PAPER_MIX` (0.5)); even rows are `surface`. A hovered row or option is mix(ground, `accent_soft`, `HOVER_ACCENT_MIX` (0.5)). The selected row is `accent_soft` with a `SELECTED_BAR_WIDTH` (3) `accent` bar at its inline start, and keeps both while hovered. The checklist's active option, shown only while the list has keyboard focus, gets an `ACTIVE_OPTION_OUTLINE` (1) `accent` outline inside the row. A focused grid draws a 2 px `accent` ring around its body.

Coloured cell text stays on grounds where the site guarantees contrast: the site keeps `accent` at 4.5:1 or better on `paper`, `surface` and `accent_soft`, and every row ground here is `surface`, a mix of `surface` and `paper`, or `accent_soft`. Food-producing goods use `ok`, gold and silver use `accent`, river bonus cells use `accent_strong`.

Scrims and shadows are black with mode-dependent alpha: scrim `SCRIM_ALPHA_LIGHT` (0.32) and `SCRIM_ALPHA_DARK` (0.6); shadow `SHADOW_ALPHA_LIGHT` (0.16) and `SHADOW_ALPHA_DARK` (0.5), blur `SHADOW_BLUR` (24), offset `SHADOW_OFFSET_Y` (8) downward. Only menus, dialogs, cards over veils and drawers have shadows.

### Palettes

The browser build receives the tokens from the parent page, resolved from computed styles, including any custom accent. Until the first message it uses dark amber. The desktop build uses these built-in palettes, copied from the site's `index.css`; an accent preset replaces only `accent`, `accent_strong` and `accent_soft`.

| Token | Light | Dark |
| --- | --- | --- |
| `paper` | `#f6f1e8` | `#000000` |
| `surface` | `#fdfaf4` | `#141210` |
| `ink` | `#211d18` | `#f0ece5` |
| `ink_muted` | `#6e655a` | `#a39a8f` |
| `ink_faint` | `#9a9084` | `#6f675d` |
| `line` | `#e3dacb` | `#262220` |
| `line_strong` | `#c9bfae` | `#3d3731` |
| `accent` (amber) | `#b45309` | `#fbbf24` |
| `accent_strong` (amber) | `#92400e` | `#fcd34d` |
| `accent_soft` (amber) | `#f6e5c4` | `#422006` |
| `ok` | `#047857` | `#34d399` |
| `danger` | `#be123c` | `#fb7185` |

| Accent preset | Light accent, strong, soft | Dark accent, strong, soft |
| --- | --- | --- |
| Lagoon | `#0f766e`, `#115e59`, `#e0f0eb` | `#2dd4bf`, `#5eead4`, `#0b2e2a` |
| Cobalt | `#1d4ed8`, `#1e3a8a`, `#dde6f8` | `#60a5fa`, `#93c5fd`, `#0f1f3d` |
| Orchid | `#7e22ce`, `#581c87`, `#ecdff7` | `#c084fc`, `#d8b4fe`, `#2a1340` |
| Graphite | `#4a443c`, `#211d18`, `#e6dfd3` | `#d6d0c6`, `#f0ece5`, `#2a2622` |

## Motion

- Palette changes (mode, accent, or any token from the host) interpolate every token from the old to the new palette over `PALETTE_MIX_MS` (90) ms with the site's chrome curve, the cubic Bezier `PALETTE_EASING_BEZIER` ([0.4, 0.0, 0.2, 1.0]), implemented as an `iced::animation::Easing::Custom` function.
- Drawers slide in from their edge and the scrim fades in over `DRAWER_SLIDE_MS` (160) ms with `Easing::EaseOutCubic`; closing reverses the same curve.
- Nothing else animates: hover and press styles, folding, sorting, menus, dialogs, scrolling and the progress bar change instantly. There is no kinetic scrolling.
- With reduced motion every duration is zero, so palette changes and drawers snap to their final state. The browser takes the flag from the site's animation switch through the host message; the desktop takes it from the Animations choice, which defaults to on. As on the site, the operating system's reduced-motion preference is not consulted.

## Direction

`ar`, `fa-IR` and `ur-PK` lay out right to left; every other locale is left to right. Direction changes only geometry; colours and metrics stay the same.

- Window: the body row order reverses, so the rail is on the right and the detail pane on the left. Splitter drag deltas change sign, and a focused splitter moves toward the arrow key pressed.
- Header: brand at the right, buttons at the left; the search field aligns its text to the right with `text_input::align_x`.
- Rail and checklists: checkboxes at the right of their labels, badges and clear buttons at the left, scrollbar on the left, the folded chevron points left, and the collapse chevron points toward the rail's own edge.
- Grid: column 0 starts at the right edge after the inset, columns run leftward, the selected-row bar is at the right, the horizontal offset is measured from the right, the vertical scrollbar is on the left, resize targets are on each column's left edge, and a positive Shift-wheel delta scrolls toward the inline end. Middle-button and touch drags follow the pointer physically in both directions: content moves with the pointer, so only the offset origin is mirrored for them. Start-aligned cells align right and numeric cells align left. ArrowLeft and ArrowRight follow the screen, so ArrowRight moves to the column visually on the right.
- Detail: label column on the right, breadcrumb separators point left, the hide chevron points toward the pane's edge.
- Menus open aligned to their opener's inline end; drawers slide in from their own inline edge.
- Numbers, identifiers, hex and RGB values, paths and the technical error detail are laid out left to right inside any direction, as on the site.

## Narrow layout and touch

Below `NARROW_BREAKPOINT` (768) the layout is narrow; the threshold matches the site's mobile limit. The desktop window keeps a minimum of 1050 by 680 (default 1440 by 900), so only the browser build reaches the narrow layout.

- Header: three rows, `NARROW_HEADER_HEIGHT` (128) in total, with `NARROW_PADDING_INLINE` (12) inline padding. Row one (`NARROW_HEADER_ROW_HEIGHT` (52)) holds three equal cells separated by `NARROW_HEADER_GAP` (8): Filters with the active-filter badge, Details and About, each `TOUCH_TARGET` (44) high; at 320 px each cell is 93 px wide. The brand is omitted, because the host page already names the application. Row two (52) is the full-width search field, 44 high with 16 px text. Row three (`NARROW_STATUS_HEIGHT` (24)) shows the result count at the inline start and the build text at the inline end in Mono 12.
- Rail and detail become overlay drawers over a `scrim`, `DRAWER_RAIL_WIDTH` (320) and `DRAWER_DETAIL_WIDTH` (360) wide, each limited to the viewport width minus `DRAWER_EDGE_GAP` (48), full window height, with a shadow and a Close button in their header. The filter drawer slides from the inline start and the detail drawer from the inline end. A tap on a row opens the detail drawer; keyboard selection does not, and Space opens it. Changing filters keeps the filter drawer open. A tap on the scrim, the Close button or Escape closes the drawer and returns focus to its opener.
- The grid fills the width. Rows are `GRID_ROW_HEIGHT_TOUCH` (40) high, checklist rows `CHECKLIST_ROW_HEIGHT_TOUCH` (40), and every button, field and icon target is at least `TOUCH_TARGET` (44). The toolbar is `NARROW_TOOLBAR_HEIGHT` (52) high and drops the legend, which moves to the bottom of the column menu. Menus and the About dialog are limited to the viewport minus `DIALOG_MARGIN` (16) on each side.
- Touch: a drag on the grid scrolls it; after `TOUCH_SLOP` (8) of movement the drag locks to its dominant axis, and movement below that is a tap that selects. Checklists scroll the same way, and a tap toggles an option. Scrollbar thumbs are `SCROLLBAR_THUMB_WIDTH_TOUCH` (4) wide. Column resizing stays available through its handles and the keyboard. There is no fling.
- In the wide layout docked widths shrink, detail first, so the grid keeps `GRID_MIN_WIDTH` (420). When both panes are at their minimum and the grid still lacks room, the detail pane collapses; showing it again collapses the rail.

## Keyboard

Tab and Shift+Tab move through every control in visual order; an open menu, dialog or drawer keeps focus inside itself until it closes. Buttons, toggles, splitters and menu items activate with Enter or Space. Text fields keep their normal editing keys.

Focus order in the wide layout:

1. Header: search, then Language and Appearance on desktop, then About.
2. Rail: collapse, Clear all, then each section in order: for each facet group its fold button, its clear button when shown, its search field when shown and its list when expanded; the toggles after Type and Raw material; the exact-colour field; each range's minimum and maximum after its group. A collapsed rail offers only its expand button.
3. Rail splitter.
4. Results: Columns, Show details when the pane is hidden, then the grid.
5. Detail splitter, then the detail pane's hide button.

The narrow order is Filters, Details, About, search, Columns, grid; the drawers hold their own order. A focused text field ignores Tab and F6, so both still move focus; it captures Escape, so the application reads Escape from `iced::event::listen_with`, which also reports captured events.

| Keys | Where | Action |
| --- | --- | --- |
| Ctrl+F (Cmd+F on macOS) | anywhere | focus the search field and select its text |
| F6, Shift+F6 | anywhere | move to the first control of the next or previous region: header, rail, results, detail |
| Escape | anywhere | close the topmost menu, dialog or drawer and return focus to its opener; a focused text field also loses focus on the same press |
| Tab, Shift+Tab | anywhere | next or previous control |
| ArrowUp, ArrowDown | grid | select the previous or next row |
| PageUp, PageDown | grid | move the selection by the visible row count minus one |
| Home, End | grid | select the first or last row |
| ArrowLeft, ArrowRight | grid | move the active column on screen and scroll it into view |
| Shift+ArrowLeft, Shift+ArrowRight | grid | move the active column's inline-end edge 16 px in the arrow's direction on screen, widening or narrowing it |
| Enter | grid | sort by the active column; again to reverse |
| Space | grid | show the detail pane or open the detail drawer for the selected row |
| ArrowUp, ArrowDown, PageUp, PageDown, Home, End | checklist, column menu | move the active option |
| Space | checklist, column menu | toggle the active option |
| ArrowLeft, ArrowRight | splitter | move the splitter by 16 px on screen |
| PageUp, PageDown, Home, End | detail region | scroll the detail content |

The active column starts at the sorted column and is drawn with a `ACTIVE_COLUMN_UNDERLINE` (2) `accent` underline in the header while the grid has focus. Selection by keyboard scrolls the row fully into view. The About dialog lists these keys.

## Limits

- No accessibility tree: iced 0.14 exposes nothing to screen readers. Keyboard operation, tooltips and fixed-size labelled controls are the mitigation.
- Clipped text: cells, headers and buttons clip; full text appears as a tooltip on headers and buttons and in the detail pane for cells.
- Desktop language, appearance and pane widths are not persisted between runs.
