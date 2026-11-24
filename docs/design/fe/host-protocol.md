# Host protocol

The browser build runs inside a sandboxed frame on the cyhdev site. This document is the contract between the parent page (`solid-csr-spa-template/src/pages/eu5_locations_db.tsx` in the cyhdev repository), the frame's host script (`web/index.html`), and the WebAssembly exports. It defines protocol version 2, which replaces the version 1 string messages `cyhdev:eu5-theme:<mode>` and `cyhdev:eu5-theme-ready`; both sides change together, and nothing accepts version 1 after the change. The desktop build has no host: the same settings come from in-app language and theme controls, which the browser build hides.

## Embedding and trust

The parent loads `/eu5-locations-db/app/index.html` in an iframe with `sandbox="allow-scripts allow-popups allow-popups-to-escape-sandbox allow-top-navigation-by-user-activation"`, and the server sends the same sandbox as a response Content-Security-Policy with `frame-ancestors 'self'`. Without `allow-same-origin` the frame document has an opaque origin, so it cannot read the parent's storage or call the site API as the viewer.

- **Parent to frame.** The parent posts with target origin `"*"`, because an opaque origin cannot be named. Messages carry only public presentation settings and the view-state string, never credentials or personal data.
- **Frame to parent.** Events arrive with `event.origin === "null"`. The parent identifies them by `event.source === frame.contentWindow` and also requires the `"null"` origin, so a same-origin page cannot impersonate the frame.
- **Frame side.** Inside the frame, `window.location.origin` is the origin of the document URL (the site origin), while the document origin is opaque. The host script accepts a message only when `event.source === window.parent` and `event.origin === window.location.origin`, and posts to the parent with `window.location.origin` as the target origin.
- **Standalone.** When the document is opened directly (`window.parent === window`), the host script posts nothing and the application runs with its defaults. View-state reporting never starts, because no host message arrives.

## Messages

Every message is a plain object sent through `postMessage` (structured clone). Receivers read only the fields named here and ignore any others, so a later minor addition does not break an older receiver. Receivers never throw: invalid input is dropped silently, and nothing is logged above debug level.

### `cyhdev:eu5-host` (parent to frame)

```json
{
  "type": "cyhdev:eu5-host",
  "v": 2,
  "theme": {
    "mode": "dark",
    "paper": "#000000", "surface": "#141210",
    "ink": "#f0ece5", "inkMuted": "#a39a8f", "inkFaint": "#6f675d",
    "line": "#262220", "lineStrong": "#3d3731",
    "accent": "#fbbf24", "accentStrong": "#fcd34d", "accentSoft": "#422006",
    "ok": "#34d399", "danger": "#fb7185"
  },
  "locale": "ko-KR",
  "dir": "ltr",
  "reducedMotion": false,
  "viewState": "s1~q.river"
}
```

| Field | Type | Allowed values | Limit |
| --- | --- | --- | --- |
| `type` | string | exactly `"cyhdev:eu5-host"` | |
| `v` | number | exactly `2` | |
| `theme.mode` | string | `"light"` or `"dark"` | |
| `theme.<token>` | string | `#rrggbb`, lowercase hexadecimal, opaque sRGB | 7 characters |
| `locale` | string | one of the 27 site tags in [Locales](#locales); any well-formed tag is accepted and unknown ones fall back | 35 characters, `^[A-Za-z]{2,3}(-[A-Za-z0-9]{2,8}){0,3}$` |
| `dir` | string | `"ltr"` or `"rtl"` | |
| `reducedMotion` | boolean | | |
| `viewState` | string, optional | a [view-state string](#view-state-string) | 2,000 characters, `^[A-Za-z0-9_.~*!-]{1,2000}$` |

The 12 colour tokens are exactly `paper`, `surface`, `ink`, `inkMuted`, `inkFaint`, `line`, `lineStrong`, `accent`, `accentStrong`, `accentSoft`, `ok` and `danger`, in that order wherever an order matters. They map to the site custom properties `--paper`, `--surface`, `--ink`, `--ink-muted`, `--ink-faint`, `--line`, `--line-strong`, `--accent`, `--accent-strong`, `--accent-soft`, `--ok` and `--danger`. Derived site tokens (`--surface-2`, `--accent-glow`, `--accent-wash`, `--grid-line`, `--glass`) are not sent; the application derives its own tints from the 12.

`reducedMotion` is `true` exactly when the site's motion switch is off. The site deliberately ignores the operating system's `prefers-reduced-motion`, so the frame does not consult it either. When it is `true`, the application applies palette changes without the 90 ms colour mix and scrolls without animation.

`dir` is the site's direction for `locale`. The application derives its layout direction from its own locale table, which agrees with the site for all 27 tags; the host script uses `dir` and `locale` only for the `lang` and `dir` attributes of the frame document, which govern the HTML error surface.

When it is sent:

- After every `cyhdev:eu5-ready` from the current frame window, with `viewState` set to the validated value of the parent URL's `view` query parameter, or without `viewState` when the parameter is absent or invalid.
- After any later change to the mode, accent, custom accent, locale or motion setting, without `viewState`. The parent sends only when the theme, locale, direction or motion value differs from the last message it sent to that frame window.
- Never before the first ready, and never in response to `cyhdev:eu5-state`.

An absent `viewState` means "leave the current view alone"; a present one replaces the whole view.

### `cyhdev:eu5-ready` (frame to parent)

```json
{ "type": "cyhdev:eu5-ready", "v": 2 }
```

Sent once per document load, after `init()` has resolved and the WebAssembly exports are callable, and after any host message that arrived earlier has been applied. It does not mean the dataset has loaded; the exports queue their values until it has. A frame reload sends a new ready, and the parent answers each one with a full host message.

### `cyhdev:eu5-state` (frame to parent)

```json
{ "type": "cyhdev:eu5-state", "v": 2, "state": "s1~q.river~k.land*sea" }
```

`state` is a canonical [view-state string](#view-state-string), at most 2,000 characters, matching `^[A-Za-z0-9_.~*!-]{1,2000}$`. The application reports only after its dataset is ready and the first host message has been applied, so a deep link is never overwritten by the default view during loading. The first report after that point is always sent; later reports are sent whenever the canonical string changes. The host script forwards them with a 250 ms trailing debounce and drops a value equal to the last one it posted.

### Validation

| Receiver | Accepts when | Otherwise |
| --- | --- | --- |
| Frame, host message | source and origin as above; `data` is a non-null, non-array object; `type`, `v`, `theme` (object with `mode` and all 12 tokens), `locale`, `dir` and `reducedMotion` all match the table | the whole message is ignored |
| Frame, `viewState` | absent, or a string matching the pattern | only `viewState` is ignored; the rest applies |
| Parent, ready | source is the frame window, origin is `"null"`, `data` is an object with the exact `type` and `v` | ignored |
| Parent, state | as for ready, and `state` is a string matching the pattern | ignored |
| WebAssembly exports | each export validates its arguments again | as defined per export in [WebAssembly interface](#webassembly-interface) |

Structured-clone data has no getters, so reading fields cannot run code; the host script still wraps every export call in `try` and ignores exceptions.

## Palette resolution in the parent

The parent resolves each token to sRGB from computed styles at the moment it sends, so presets, the custom accent and any future `color-mix()` definition all arrive as plain colours.

1. The mode is read from the root element (`document.documentElement.classList.contains("dark")`), not from the theme signal, so it always matches the colours being read.
2. A hidden probe element (`display: none` keeps computed styles) is appended inside `document.body`, so it inherits the root's custom properties, including `data-accent` presets and the inline `--custom-accent-*` properties. For each token its `style.color` is set to `var(--token)`, and `getComputedStyle(probe).color` is read. The probe is removed afterwards.
3. The computed text is normalized to `#rrggbb`. Browsers return `rgb(r, g, b)` for sRGB values; a `color-mix()` or wide-gamut value can come back as `oklab(...)` or `color(...)`, which is converted by painting one pixel with that `fillStyle` on a 1 by 1 two-dimensional canvas and reading it back with `getImageData`, which returns sRGB bytes.
4. A token that cannot be resolved, or resolves to a colour with alpha below 1, takes the built-in default for the mode (the amber values in `index.css`), so the message is always complete.
5. Sends are triggered by a `MutationObserver` on the root element's `class`, `data-accent`, `style`, `lang`, `dir` and `data-motion` attributes, coalesced in one microtask. Observing the DOM instead of the signals avoids reading stale styles, because the theme class and accent attributes are applied after the signals change.

## Frame host script

`web/index.html` is the only file in the frame besides the generated `pkg/` and the lazily fetched packs.

### Elements

| Id | Element | Purpose |
| --- | --- | --- |
| `canvas` | `<canvas>` | Placeholder named by `window::Settings.platform_specific.target = Some("canvas")`; iced replaces it with its own canvas, which has no id, so styles use the `canvas` element selector |
| `eu5-error` | `<div role="alert" hidden>` | Error surface, outside the canvas so the replacement cannot remove it |
| `eu5-error-text` | `<p>` inside `eu5-error` | Error message text |

The document sets `<meta name="color-scheme" content="light dark">` and leaves the page background transparent until the first host message, so the parent's paper colour shows through while the application loads. Each applied host message sets `color-scheme` to the mode and the background to `theme.paper` on the root element, which keeps the browser from painting an opaque backdrop behind a frame whose colour scheme differs from the parent's.

### Startup order

1. Define `globalThis.cyhdevEu5ReportState` and `globalThis.cyhdevEu5FetchAsset` (below). They must exist before `init()`, because the application may call them during startup.
2. Register the `message` listener. Until the exports are callable it validates and keeps only the latest valid host message.
3. Check for a renderer: `navigator.gpu.requestAdapter()` resolving to an adapter within 3 seconds, or else a non-null `getContext("webgl2")` on a detached canvas, whose context is released through `WEBGL_lose_context`. A `navigator.gpu` object alone does not prove an adapter exists.
4. If neither renderer is available, show `eu5-error` with "This viewer needs WebGPU or WebGL 2, and this browser provides neither." and stop: no `init()`, no ready.
5. `await init()` from `./pkg/eu5_location_filter.js`. The start function hands the event loop to the browser and returns. A rejection whose message contains "Using exceptions for control flow" counts as success; any other rejection shows "The viewer could not start. Reload the page to try again." and stops.
6. Mark the exports callable and apply the buffered host message, if any.
7. Unless standalone, post `cyhdev:eu5-ready`.

The error texts are English only and the frame keeps `lang="en"` and `dir="ltr"` in that case, because a failed frame never receives a host message.

Applying a host message calls the exports in this fixed order: `set_locale`, `set_reduced_motion`, `apply_view_state` (only when `viewState` is present and valid), then `set_palette`. The first `set_palette` call marks the host as connected; placing it last guarantees that an initial view state is queued before reporting can begin.

### Imported functions

The application imports two plain globals instead of `inline_js`, so `wasm-pack` emits no `pkg/snippets/` files and staging keeps its fixed file list.

| Global | Signature | Behaviour |
| --- | --- | --- |
| `cyhdevEu5ReportState` | `(state: string) => void` | Validates length and charset, debounces for 250 ms, drops a repeat of the last posted value, and posts `cyhdev:eu5-state`. Synchronous, never throws, never calls back into WebAssembly. A no-op when standalone. |
| `cyhdevEu5FetchAsset` | `(path: string, maxBytes: number) => Promise<Uint8Array>` | Fetches one pack relative to the document and resolves its bytes; rejects with an `Error` on any failure. Never calls back into WebAssembly. |

### Lazily fetched assets

Language and font packs live next to `index.html` and are addressed relative to the document: `cyhdevEu5FetchAsset` resolves `new URL(path, document.baseURI)`, so on cyhdev `lang/ko.26547371.bitcode.zst` becomes `/eu5-locations-db/app/lang/ko.26547371.bitcode.zst`.

- **Paths.** `path` must match `^(lang/[a-z-]{2,16}\.[0-9]{1,20}\.bitcode\.zst|fonts/[a-z0-9-]{1,48}\.ttf)$`; anything else rejects without a request.
- **Language packs.** `lang/<code>.<build>.bitcode.zst`, where `<code>` is the pack code of a game language from [Locales](#locales) (`GameLanguage::code` in `src/lang_pack/language.rs`) and `<build>` is the decimal Steam build ID of the embedded dataset. The application still rejects a pack whose header build or location count differs from the embedded dataset.
- **Font packs.** `fonts/<name>.ttf`, with `<name>` from [Locales](#locales). Names are stable; the site serves these files with revalidation (`max-age=0, must-revalidate`), because they are not in the frontend build manifest.
- **Request.** `fetch(url, { mode: "cors", credentials: "omit" })`. The opaque-origin document fetches in CORS mode and the server answers with `Access-Control-Allow-Origin: *`.
- **Bounds.** Rejects a non-2xx status, a `Content-Length` above `maxBytes`, or a body above `maxBytes` after reading. `maxBytes` is at most 33,554,432; the application passes 16,777,216 for both pack kinds, which is `MAX_PACK_COMPRESSED_SIZE` for language packs.

### Search field

The canvas receives no input-method composition and no clipboard in the browser (winit's web backend emits no IME events and its clipboard is a placeholder), so Korean, Japanese and Chinese text could not be typed or pasted into an iced text field. The global search in the browser build is therefore a real `<input id="eu5-search" type="search" autocomplete="off" spellcheck="false">` placed over the header's search slot. This is internal to the frame: the parent page never sees it, and it adds no message to the protocol. Facet option searches and numeric fields stay iced text fields, and the desktop build keeps an iced text field for the global search.

- **Placement.** The application measures the header slot and calls `cyhdevEu5SearchOverlay(visible, x, y, width, height, text, placeholder, rtl)` with CSS pixels of the frame document, which equal iced logical pixels because the canvas fills the document. It calls with `visible` false while the dataset is loading or failed, while a modal surface (About, a drawer with a scrim, the loading or error veil) covers the header, and before the slot has been laid out. It skips the call when nothing changed since the previous one.
- **Content.** The host script sets position, size, placeholder, accessible label and direction on every visible call, and sets `value` only when it differs and no composition is in progress, so the application never overwrites text the viewer is composing.
- **Input.** Every `input` event, and the end of a composition, truncates the value to 256 UTF-8 bytes on a code point boundary and calls the export `set_search_text(text) -> bool`, which stores the text in its own inbox slot. The application applies it exactly like typing into the desktop field, so search results update while the viewer types.
- **Focus.** Ctrl+F or Cmd+F in the browser calls `cyhdevEu5FocusSearch()`, which focuses the field and selects its text after the current call returns. Escape in the field blurs it and returns focus to the canvas.
- **Look.** The field starts hidden and is styled from the last valid host message: surface background, ink text, line border and an accent focus outline, with the built-in dark values before any message, in `14px system-ui`.

Both globals are defined before `init()`, never throw, and never call back into WebAssembly synchronously; `set_search_text` is called only from DOM events, outside any application update.

## WebAssembly interface

Exports are `#[wasm_bindgen]` functions compiled only for `feature = "web"` on `target_family = "wasm"`; their JavaScript names equal the Rust names. None returns a `Result`, so none throws into the host script.

| Rust signature | Called with | Validation and result |
| --- | --- | --- |
| `pub fn set_palette(mode: &str, colours: &[u32]) -> bool` | `theme.mode` and a `Uint32Array` of the 12 tokens in protocol order, each `0xRRGGBB` | `mode` is `"light"` or `"dark"`, exactly 12 values, each at most `0xFFFFFF`; otherwise `false` and no change. Always marks the host as connected. |
| `pub fn set_locale(tag: &str) -> bool` | `locale` | `true` for one of the 27 tags (exact, case-sensitive). Any other tag applies `en-US` and returns `false`. |
| `pub fn set_reduced_motion(reduced: bool)` | `reducedMotion` | No validation needed. |
| `pub fn apply_view_state(text: &str) -> bool` | `viewState` | `false` and no change when the string is empty, longer than 2,000 characters, outside the charset, or not version `s1`. Otherwise `true`; field and item errors drop only that field or item. |

Imports, declared in an `extern "C"` block without a `module` attribute so the generated glue refers to them as globals. The host script defines both before `init()`, so the lookup succeeds whether the glue resolves them at instantiation or at call time:

```rust
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_name = cyhdevEu5ReportState)]
    fn report_view_state(state: &str) -> Result<(), JsValue>;

    #[wasm_bindgen(catch, js_name = cyhdevEu5FetchAsset)]
    fn fetch_asset(path: &str, max_bytes: u32) -> Result<js_sys::Promise, JsValue>;
}
```

`catch` turns a missing global or a thrown exception into `Err`, which the application ignores for reports and treats as a failed load for assets.

**Before the application has loaded.** The exports can run before the iced application has booted and before the dataset has decoded. Each one stores its validated value in a fixed slot (palette, locale, motion, view state), where a later call replaces an earlier one, and wakes the application's host subscription. Palette, locale and motion are read once at boot, so the first frame already uses them, and then on every wake. A queued view state is applied once the dataset and its facet option lists are ready, replacing the whole filter, sort and selection state. The slots hold at most one value each, so nothing grows with the number of messages.

**Reporting.** The application calls `report_view_state` from its update loop, never from inside an export, once reporting is enabled (dataset ready and host connected) and whenever the canonical string differs from the last one it reported.

## View-state string

The view state is a compact, URL-safe, versioned text form of the search, facet selections, numeric ranges, exact colour, flags, sort and selected location. It uses only `A-Z a-z 0-9 _ - . ~ * !`, so it needs no percent-encoding in a query component and survives `encodeURIComponent` unchanged. It never contains numeric dataset IDs: facets and the selection use stable game identifiers, so a link survives a re-import.

### Grammar

```ebnf
view-state  = version , { "~" , field } ;
version     = "s1" ;
field       = name , "." , value ;            (* the first "." ends the name *)
name        = lower , { lower } ;              (* 1 to 8 letters *)
value       = { raw | "*" | "!" } ;            (* interpreted per field, below *)

text        = { raw | escape } ;
escape      = "!" , hex , hex ;                (* one UTF-8 byte *)
raw         = upper | lower | digit | "_" | "-" | "." ;
list        = [ item , { "*" , item } ] ;      (* empty value = empty list *)
symbol-item = "!" | text ;                     (* a lone "!" is the missing value *)
number      = [ "-" ] , digit , { digit } , [ "." , digit , { digit } ] ;
range       = [ number ] , "*" , [ number ] ;  (* minimum, maximum; inclusive *)
rgb         = 6 * hex ;
flag        = "0" | "1" ;
sort        = sort-column , "*" , ( "a" | "d" ) ;
hex         = digit | "A" | "B" | "C" | "D" | "E" | "F"
            | "a" | "b" | "c" | "d" | "e" | "f" ;
```

**Escaping.** Text (search, symbol keys, the selected key) is encoded as UTF-8. Bytes in `raw` are written as themselves; every other byte, including space, `~`, `*`, `!` and all non-ASCII bytes, is written as `!` followed by two uppercase hexadecimal digits. Decoders accept either hex case and escaped raw bytes, and drop an item whose escape is incomplete or whose bytes are not valid UTF-8. A lone `!` cannot be the encoding of any text, which is why it can stand for the missing value.

### Fields

Fields appear in this order. A field equal to its default is omitted.

| # | Name | Meaning | Value | Default |
| --- | --- | --- | --- | --- |
| 1 | `q` | search text | `text`, 1 to 256 bytes decoded, no U+0000 to U+001F or U+007F | empty |
| 2 | `k` | location kinds | `list` of `land`, `sea`, `lake`, `impassable`, `unknown` | exactly `land`; `k.` is the empty set (every kind) |
| 3 | `tp` | topography | `list` of `text` keys; the missing value is not allowed | empty |
| 4 | `vg` | vegetation | `list` of `symbol-item` | empty |
| 5 | `cl` | climate | `list` of `symbol-item` | empty |
| 6 | `ct` | continent | `list` of `text` keys; the missing value is not allowed | empty |
| 7 | `sc` | subcontinent | `list` of `text` keys; the missing value is not allowed | empty |
| 8 | `rg` | region | `list` of `text` keys; the missing value is not allowed | empty |
| 9 | `ar` | area | `list` of `text` keys; the missing value is not allowed | empty |
| 10 | `pv` | province | `list` of `text` keys; the missing value is not allowed | empty |
| 11 | `rl` | religion | `list` of `symbol-item` | empty |
| 12 | `cu` | culture | `list` of `symbol-item` | empty |
| 13 | `rm` | raw material | `list` of `symbol-item` | empty |
| 14 | `fo` | food-producing raw materials only | `flag`; written as `1` | `0` |
| 15 | `md` | modifier | `list` of `symbol-item` | empty |
| 16 | `c` | exact map colour | `rgb`; written uppercase | none |
| 17 | `co` | coastal | `list` of `yes`, `no` | empty |
| 18 | `rp` | river presence | `list` of `present`, `missing` | empty |
| 19 | `lm` | minimum river bonus tier | `list` of decimal integers 1 to 255 without leading zeros | empty |
| 20 | `lx` | maximum river bonus tier | as `lm` | empty |
| 21 | `hp` | harbor presence | `list` of `present`, `missing` | empty |
| 22 | `hr` | harbor suitability | `range` | both bounds empty |
| 23 | `mp` | movement-assistance presence | `list` of `present`, `missing` | empty |
| 24 | `mx` | first movement component | `range` | both bounds empty |
| 25 | `my` | second movement component | `range` | both bounds empty |
| 26 | `si` | impassable locations shown | `flag`; written as `0` | `1` |
| 27 | `s` | sort | `sort` | `name*a` |
| 28 | `l` | selected location | `text` key, 1 to 128 bytes decoded | none |

Symbol keys are the game's internal identifiers as stored in the dataset dictionary (for example `hills` for a topography), 1 to 128 bytes decoded. `sort-column` is one of `color`, `name`, `identifier`, `kind`, `topography`, `vegetation`, `climate`, `continent`, `subcontinent`, `region`, `area`, `province`, `religion`, `culture`, `raw_material`, `modifier`, `coastal`, `river_presence`, `river_level`, `harbor`, `movement_presence`, `movement_x`, `movement_y`, `static_capacity` or `equator_capacity`, the same keys the result columns use; decoders also accept `rgb` as an alias of `color`.

### Canonical form

The encoder output is unique for a given state, so equal states produce equal strings and the parent's URL changes only when the view does.

- Fields appear in table order, each at most once, defaults omitted.
- List items are deduplicated and sorted by the byte order of their encoded text; a lone `!` therefore sorts first.
- Numbers use the shortest decimal that parses back to the same `f32`, without exponent, as Rust's `f32` `Display` writes it; negative zero is written `0`. A range is written `min*max` with an empty side for a missing bound.
- Colours use six uppercase hexadecimal digits, flags use the single non-default digit, and escapes use uppercase hex.
- The encoder omits any field or item its own decoder would reject: a search longer than 256 bytes or containing a control character, a key longer than 128 bytes, a non-finite bound. Such a state still applies inside the application; only the link leaves that part out. This keeps `decode(encode(state))` equal to the encodable part of the state.
- When the canonical string would exceed 2,000 characters (for example several hundred selected provinces), the application reports the default `s1` instead, so the parent drops the deep link rather than keep a stale one.

### Decoding

- The whole string is rejected, leaving the view unchanged, when it is empty, exceeds 2,000 characters, contains a character outside the charset, or its first token is not `s1`. A future `s2` is therefore ignored by an older application, which then reports its own state.
- Tokens are split on `~`, and each field on its first `.`; a token without `.` is dropped. At most 64 fields and 1,000 items per list are read; later ones are ignored.
- Unknown field names are ignored, which is the forward-compatibility rule: a later version adds fields without changing `s1`, and older decoders skip them. A field repeated later in the string is ignored; the first occurrence wins.
- An invalid field value (bad flag, malformed number, number longer than 48 characters, non-finite number, wrong colour length, unknown sort column or direction) drops that field, which then takes its default.
- An unknown list item (a key not in that facet's options for the loaded dataset, an unknown kind or presence word, a missing value in `tp`, `ct`, `sc`, `rg`, `ar` or `pv`, a river tier outside 1 to `MAX_RIVER_LEVEL`) drops that item. If every item of a field is dropped, the field takes its default.
- `l` applies only when the key names a location in the filtered result; otherwise nothing is selected.
- Absent fields take their defaults: applying a view state replaces the whole filter, sort and selection state.

### Examples

| State | String |
| --- | --- |
| Default: no search, land only, impassable shown, sorted by name ascending, nothing selected | `s1` |
| Search "river", land and sea, sorted by harbor suitability descending | `s1~q.river~k.land*sea~s.harbor*d` |
| Search "new york", hills or mountains, climate missing, impassable hidden | `s1~q.new!20york~tp.hills*mountains~cl.!~si.0` |
| Search "서울", colour 1A2B3C, coastal, minimum river tier 2 or 3, harbor at least 0.5 | `s1~q.!EC!84!9C!EC!9A!B8~c.1A2B3C~co.yes~lm.2*3~hr.0.5*` |
| Every kind, raw material missing or `wheat`, food-producing only, first movement component -1.5 to 2, sorted by name descending, `stockholm` selected | `s1~k.~rm.!*wheat~fo.1~mx.-1.5*2~s.name*d~l.stockholm` |

The keys `wheat` and `stockholm` illustrate the syntax; real links use whatever identifiers the dataset holds. A non-canonical input such as `s1~k.sea*land~zz.9~q.river~q.lake` decodes to search "river" with land and sea (the unknown `zz` and the repeated `q` are ignored), and the application then reports `s1~q.river~k.land*sea`.

### Parent URL

The parent mirrors the state into the `view` query parameter of its own page URL, for example `/eu5-locations-db?view=s1~q.river~k.land*sea`.

- The parent treats the string as opaque. It checks only the charset and length pattern, and compares it with the default `s1`, which removes the parameter instead of writing it.
- Every update replaces the current history entry; the page never pushes entries for view changes, so Back leaves the page instead of stepping through filters.
- Other query parameters and the fragment are preserved. The `view` parameter is written last and unencoded, because its charset needs no encoding; it is read back with `URLSearchParams`, which also decodes a percent-encoded copy.
- On each ready, the parent sends the current `view` value if it passes validation. An invalid value is not sent, and the application's first report then replaces it.

## Locales

The application accepts the 27 site tags below. Direction mirrors the layout for `ar`, `fa-IR` and `ur-PK`. Game text comes from the embedded English data or a lazily fetched language pack, named in the table by its install directory and pack code; the font is the embedded IBM Plex subset (Latin, Latin Extended including Vietnamese, Cyrillic) or a lazily fetched font pack.

| Tag | Direction | Game language (directory, pack code) | Font pack |
| --- | --- | --- | --- |
| `en-US` | LTR | `english`, embedded | embedded |
| `fr-FR` | LTR | `french`, `fr` | embedded |
| `de-DE` | LTR | `german`, `de` | embedded |
| `es-ES` | LTR | `spanish`, `es` | embedded |
| `pt-BR` | LTR | `braz_por`, `pt-br` | embedded |
| `pl-PL` | LTR | `polish`, `pl` | embedded |
| `tr-TR` | LTR | `turkish`, `tr` | embedded |
| `ru-RU` | LTR | `russian`, `ru` | embedded |
| `it-IT`, `sv-SE`, `nl-NL`, `ro-RO`, `id-ID`, `tl-PH`, `sh`, `vi-VN` | LTR | `english` | embedded |
| `uk-UA` | LTR | `english` | embedded |
| `ko-KR` | LTR | `korean`, `ko` | `plex-sans-kr` |
| `ja-JP` | LTR | `japanese`, `ja` | `plex-sans-jp` |
| `zh-Hans` | LTR | `simp_chinese`, `zh-hans` | `sans-sc` |
| `zh-Hant` | LTR | `english` | `sans-tc` |
| `ar`, `fa-IR`, `ur-PK` | RTL | `english` | `noto-sans-arabic` |
| `hi-IN` | LTR | `english` | `noto-sans-devanagari` |
| `bn-BD` | LTR | `english` | `noto-sans-bengali` |
| `te-IN` | LTR | `english` | `noto-sans-telugu` |

The directory names are the EU5 1.4 localization directories (`braz_por`, `french`, `german`, `japanese`, `korean`, `polish`, `russian`, `simp_chinese`, `spanish`, `turkish`, plus embedded `english`); the pack codes `fr`, `de`, `es`, `pt-br`, `pl`, `ru`, `tr`, `ja`, `ko` and `zh-hans` appear only in pack file names, and `src/lang_pack/language.rs` keeps both in one table. One non-English pack is held in memory at a time. A pack or font that fails to load leaves English game text or the embedded font in place, and the interface stays usable.

An unknown tag (anything not in the table, compared exactly) selects `en-US`: English interface text, English game text, the embedded font and left-to-right layout. `set_locale` returns `false` in that case, and the next valid tag replaces it.

## Parent page module

The parent page exports these names from `pages/eu5_locations_db.tsx`, next to the existing `calculateViewportOffsets`. Everything except `createEu5ColorReader` is pure and unit tested with Vitest; the component wires them to `postMessage`, the `MutationObserver` and the router.

```ts
export const EU5_PROTOCOL_VERSION = 2;
export const EU5_HOST_MESSAGE_TYPE = "cyhdev:eu5-host";
export const EU5_READY_MESSAGE_TYPE = "cyhdev:eu5-ready";
export const EU5_STATE_MESSAGE_TYPE = "cyhdev:eu5-state";
export const EU5_VIEW_STATE_PARAM = "view";
export const EU5_VIEW_STATE_MAX_LENGTH = 2000;
export const EU5_DEFAULT_VIEW_STATE = "s1";
export const EU5_VIEW_STATE_PATTERN = /^[A-Za-z0-9_.~*!-]{1,2000}$/;

export const EU5_PALETTE_TOKENS = [
  "paper", "surface", "ink", "inkMuted", "inkFaint", "line", "lineStrong",
  "accent", "accentStrong", "accentSoft", "ok", "danger",
] as const;
export type Eu5PaletteToken = (typeof EU5_PALETTE_TOKENS)[number];
export type Eu5ThemeMode = "light" | "dark";
export type Eu5Palette = Readonly<Record<Eu5PaletteToken, string>>;
export type Eu5HostTheme = Readonly<{ mode: Eu5ThemeMode }> & Eu5Palette;

/** Site custom property for each token, such as inkMuted to --ink-muted. */
export const EU5_PALETTE_PROPERTIES: Readonly<Record<Eu5PaletteToken, `--${string}`>>;
/** The amber values from index.css, used for any token that cannot be resolved. */
export const EU5_DEFAULT_PALETTES: Readonly<Record<Eu5ThemeMode, Eu5Palette>>;

export type Eu5HostMessage = Readonly<{
  type: typeof EU5_HOST_MESSAGE_TYPE;
  v: typeof EU5_PROTOCOL_VERSION;
  theme: Eu5HostTheme;
  locale: UiLocale;
  dir: "ltr" | "rtl";
  reducedMotion: boolean;
  viewState?: string;
}>;
export type Eu5MessageEvent = Pick<MessageEvent<unknown>, "source" | "origin" | "data">;

/** #rrggbb from hex, rgb(), rgba() or color(srgb ...) text with alpha 1; otherwise null. */
export const normalizeCssColor: (value: string) => string | null;
/** All 12 tokens through readColor and normalizeCssColor, defaulting per token. */
export const resolveEu5Theme: (
  mode: Eu5ThemeMode,
  readColor: (property: string) => string | null,
) => Eu5HostTheme;
/** DOM adapter: probe element, computed style, canvas fallback. Not pure. */
export const createEu5ColorReader: (doc: Document) => (property: string) => string | null;
export const isEu5ViewState: (value: unknown) => value is string;
/** Derives dir with localeDirection; includes viewState only when it passes isEu5ViewState. */
export const buildEu5HostMessage: (input: Readonly<{
  theme: Eu5HostTheme;
  locale: UiLocale;
  reducedMotion: boolean;
  viewState?: string | null;
}>) => Eu5HostMessage;
/** True when theme, locale, direction and motion are equal; viewState is not compared. */
export const sameEu5HostSettings: (a: Eu5HostMessage, b: Eu5HostMessage) => boolean;
export const isEu5Ready: (event: Eu5MessageEvent, frameWindow: MessageEventSource | null) => boolean;
/** The validated state of a state message from the frame, or null. */
export const readEu5StateMessage: (
  event: Eu5MessageEvent,
  frameWindow: MessageEventSource | null,
) => string | null;
/** The validated view parameter of a location.search string, or null. */
export const readEu5ViewStateParam: (search: string) => string | null;
/**
 * search with view removed, then appended unencoded unless state is null or
 * the default. Returns "" or a string starting with "?".
 */
export const withEu5ViewStateParam: (search: string, state: string | null) => string;
```

The component writes the URL with the router's `navigate(pathname + search + hash, { replace: true, scroll: false })`, and only when the new search string differs from the current one, so the router's location stays in step with the address bar.
