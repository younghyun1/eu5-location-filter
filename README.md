# EU5 Location Filter

[![CI](https://github.com/younghyun1/eu5-location-filter/actions/workflows/ci.yml/badge.svg)](https://github.com/younghyun1/eu5-location-filter/actions/workflows/ci.yml)

EU5 Location Filter is an unofficial desktop and browser application for the static map locations in Europa Universalis V 1.4.8. The interface is written with [iced](https://iced.rs) 0.14 and follows the cyhdev site's palette, language, text direction and motion. Both targets embed compressed English location data and precomputed search and sort indexes for Steam build `26547371`, then decompress both into memory at startup without reading a game installation. Location names and labels in ten more game languages come from separate language packs, and the interface text is available in the site's 27 locales.

## Install and run

Install from a source checkout with the Rust 1.100 nightly toolchain or newer:

```text
cargo install --path . --locked
eu5-location-filter
```

Normal GUI startup uses the embedded bundles. `--data-file PATH` and `--index-file PATH` override them with external bundles. Maintainers can regenerate the committed assets from a local Steam installation with `eu5-location-filter import --force`; `--game-dir PATH` selects an explicit installation, `--languages` limits the language packs, and `--lang-dir` sets the pack directory (default `assets/lang`). `eu5-location-filter reindex --force` regenerates only the query indexes and the language packs from the existing location blob, without discovering or reading EU5 files. `eu5-location-filter diff --old PATH --new PATH` reports the changes between two bundles, including localized text when language directories are given.

The browser package is built with `wasm-pack --target web` and renders the same interface with wgpu into a canvas that replaces the placeholder `canvas` element of `web/index.html`. It uses WebGPU and falls back to WebGL 2; with neither, the host page shows a message instead of starting. Inside the cyhdev site it runs in a sandboxed frame and follows the site through the versioned message protocol described in [host protocol](docs/design/fe/host-protocol.md): the twelve palette colours, the locale, the text direction, reduced motion and a shareable view-state string that the site mirrors into the page address. See [development](docs/development.md) for the build commands. Steam discovery, external files, retries, data rebuilding and the in-app Language and Appearance menus are desktop-only.

## Languages and fonts

The interface text is written for the 27 locales of the cyhdev site, and Arabic, Persian and Urdu mirror the whole layout. Game text follows the interface language where EU5 ships it: French, German, Spanish, Brazilian Portuguese, Polish, Russian, Turkish, Japanese, Korean and Simplified Chinese have a language pack with localized names and labels, their own search text and their own sort orders; every other locale shows English game text. The desktop executable embeds all ten packs. The browser downloads a pack only when its language is selected, holds one at a time, and keeps English if the download fails. Searching a localized name also matches the English name and the identifier.

IBM Plex Sans and IBM Plex Mono are embedded and cover Latin, Latin Extended, Vietnamese and Cyrillic. Korean, Japanese, Chinese, Arabic, Devanagari, Bengali and Telugu text uses a font pack that the desktop executable embeds and the browser downloads on demand. The catalogs for 25 locales other than English and Korean are drafts that need review by a native reader. Licence texts are in `assets/licenses/` and in the About dialog. The interface text catalogs are described in `assets/i18n/README.md`.

## Accessibility and known limits

iced 0.14 exposes no accessibility tree, so screen readers cannot read this interface. Keyboard operation is complete instead: Tab and Shift+Tab visit every control, F6 moves between regions, Ctrl+F (Cmd+F on macOS) focuses the search field, and the result grid and every checklist work with the arrow, Page, Home, End and Space keys. The full table is in [layout](docs/design/fe/layout.md) and the About dialog. Further limits:

- Text that does not fit is clipped, not elided; the full text is shown as a tooltip or in the detail pane.
- Sorting uses code-point order after case and diacritic folding, not a locale collation: kanji and hanzi sort by code point, not by reading.
- Russian and Polish culture labels are the stems the game completes at run time and are shown as shipped.
- In the browser the global search is a real text field placed over the canvas, because the canvas receives no input-method composition or clipboard; the facet search fields and numeric fields are canvas text fields and do not accept input-method composition or paste in the browser.
- In the browser, Tab does not move focus out of the canvas, because the key press is consumed by the canvas.
- Desktop language, appearance and pane widths are not kept between runs.

The committed bundles contain static names and map metadata derived from the installed game. They do not contain the original Paradox text files or map images. The bundle headers record the represented build, and both files are included in the source repository and compiled executable.

## Supported release targets

Tagged releases are configured for native Windows, Linux, and macOS runners on x86-64 and ARM64. x86-64 artifacts require the `x86-64-v3` feature level associated with Haswell-era processors. ARM64 artifacts use Rust's generic AArch64 CPU, retaining the architecture's ARMv8-A baseline instead of tuning for the build runner. Release publication is gated on Windows Authenticode signing, Apple Developer ID signing and notarization, detached GPG signatures, checksums, and GitHub provenance attestations; see [release signing](docs/signing.md).

## Static population capacity

The committed dataset stores a population-capacity estimate derived only from immutable EU5 1.4.8 map data. It combines vegetation capacity and the location's map-coordinate distance from `equator_y`, then applies topography, climate, coastal, and five-tier river modifiers. Development, staffed buildings, location rank, laws, societal values, country modifiers, and other mutable campaign state are deliberately excluded. The total and the equator contribution are sortable columns, and the selected-location pane shows the full static breakdown.

## Filters

The application starts with Type set to Land. Sea, lake, impassable, and unknown types remain selectable; impassability includes `*_wasteland` and `salt_pans` topographies plus EU5's explicit `impassable_mountains` map list. Name, identifier, and option searches ignore case, common Latin diacritics, punctuation, and whitespace, so transliterations remain searchable without reproducing their separators exactly. Categorical filters use bounded searchable checkbox lists with non-exclusive OR selections inside a field, including a reversible food-producing raw-material restriction; numeric ranges and exact RGB remain validated inputs; the filter state, sort and selected location can be encoded in the view-state string and restored from it. Different fields combine with AND, and empty fields mean any value. Invalid numeric or RGB input is shown inline and does not change the active result set. The result table has sortable, resizable columns, a toggled column-visibility menu covering every filter criterion and static capacity value, a default raw-material column with green food-producing labels, amber gold and silver labels, and draggable pane boundaries. Both the filter rail and detail pane can be hidden without discarding their expanded widths. Below 768 logical pixels, which only the browser build reaches, the rail and the detail pane become drawers, the header stacks, and rows and controls use touch-sized targets.

## Scope and license

The importer reads only vanilla installation files. Mods, saves, ownership, population, and other campaign state are out of scope. Europa Universalis and Paradox Interactive are trademarks of their respective owners. This project is unofficial and is not affiliated with or endorsed by Paradox Interactive.

The source is available under MIT OR Apache-2.0. The interface uses iced under the MIT licence, and the embedded and downloadable fonts (IBM Plex and Noto) are under the SIL Open Font License 1.1; the licence texts are in `assets/licenses/` and in the About dialog.

Created by Young Hyun Chi. Project links: [GitHub](https://github.com/younghyun1), [cyhdev.com](https://cyhdev.com), and [LinkedIn](https://www.linkedin.com/in/young-hyun-chi-553431376/).
