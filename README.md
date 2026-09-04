# EU5 Location Filter

[![CI](https://github.com/younghyun1/eu5-location-filter/actions/workflows/ci.yml/badge.svg)](https://github.com/younghyun1/eu5-location-filter/actions/workflows/ci.yml)

EU5 Location Filter is an unofficial desktop and browser application for the static map locations in Europa Universalis V 1.3.11. Both targets embed compressed location data and precomputed search and sort indexes for Steam build `24187685`, then decompress both into memory at startup without reading a game installation.

## Install and run

Install from a source checkout with the Rust 1.100 nightly toolchain or newer:

```text
cargo install --path . --locked
eu5-location-filter
```

Normal GUI startup uses the embedded bundles. `--data-file PATH` and `--index-file PATH` override them with external bundles. Maintainers can regenerate both committed assets from the local Windows Steam installation with `eu5-location-filter import --force`; `--game-dir PATH` remains available for explicit installations.

The browser package is built with `wasm-pack --target web` and renders the same Slint interface into `canvas#canvas`. Its host document accepts the bounded `cyhdev:eu5-theme:light` and `cyhdev:eu5-theme:dark` same-origin messages, applying the website's semantic cream, black, warm-neutral, and amber palette without reloading the application. See [development](docs/development.md) for the development command. Steam discovery, external files, retries, and data rebuilding remain desktop-only.

The committed bundles contain static names and map metadata derived from the installed game. They do not contain the original Paradox text files or map images. The bundle headers record the represented build, and both files are included in the source repository and compiled executable.

## Supported release targets

Tagged releases are configured for native Windows, Linux, and macOS runners on x86-64 and ARM64. x86-64 artifacts require the `x86-64-v3` feature level associated with Haswell-era processors. ARM64 artifacts use Rust's generic AArch64 CPU, retaining the architecture's ARMv8-A baseline instead of tuning for the build runner. Release publication is gated on Windows Authenticode signing, Apple Developer ID signing and notarization, detached GPG signatures, checksums, and GitHub provenance attestations; see [release signing](docs/signing.md).

## Static population capacity

The committed dataset stores a population-capacity estimate derived only from immutable EU5 1.3.11 map data. It combines vegetation capacity and the location's map-coordinate distance from `equator_y`, then applies topography, climate, coastal, and five-tier river modifiers. Development, staffed buildings, location rank, laws, societal values, country modifiers, and other mutable campaign state are deliberately excluded. The total and the equator contribution are sortable columns, and the selected-location pane shows the full static breakdown.

## Filters

The application starts with Type set to Land. Sea, lake, impassable, and unknown types remain selectable; impassability includes `*_wasteland` and `salt_pans` topographies plus EU5's explicit `impassable_mountains` map list. Name, identifier, and option searches ignore case, common Latin diacritics, punctuation, and whitespace, so transliterations remain searchable without reproducing their separators exactly. Categorical filters use bounded searchable checkbox lists with non-exclusive OR selections inside a field, including a reversible food-producing raw-material restriction; numeric ranges and exact RGB remain validated inputs. Different fields combine with AND, and empty fields mean any value. Invalid numeric or RGB input is shown inline and does not change the active result set. The result table has sortable, resizable columns, a toggled column-visibility menu covering every filter criterion and static capacity value, a default raw-material column with unique glyphs, green food-producing raw materials, amber gold and silver, and draggable pane boundaries. Both the filter rail and detail pane can be hidden without discarding their expanded widths.

## Scope and license

The importer reads only vanilla installation files. Mods, saves, ownership, population, and other campaign state are out of scope. Europa Universalis and Paradox Interactive are trademarks of their respective owners. This project is unofficial and is not affiliated with or endorsed by Paradox Interactive.

The source is available under MIT OR Apache-2.0. The UI uses Slint under the Slint Royalty-free License and includes Slint attribution in its About dialog.

Created by Young Hyun Chi. Project links: [GitHub](https://github.com/younghyun1), [cyhdev.com](https://cyhdev.com), and [LinkedIn](https://www.linkedin.com/in/young-hyun-chi-553431376/).
