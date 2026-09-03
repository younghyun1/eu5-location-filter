# EU5 Location Filter

EU5 Location Filter is an unofficial desktop browser for the static map locations in Europa Universalis V 1.3.11. The executable embeds compressed location data and precomputed search and sort indexes for Steam build `24187685`, then decompresses both into memory at startup without reading a game installation.

## Install and run

Install from a source checkout with the Rust 1.100 nightly toolchain or newer:

```text
cargo install --path . --locked
eu5-location-filter
```

Normal GUI startup uses the embedded bundles. `--data-file PATH` and `--index-file PATH` override them with external bundles. Maintainers can regenerate both committed assets from the local Windows Steam installation with `eu5-location-filter import --force`; `--game-dir PATH` remains available for explicit installations.

The committed bundles contain static names and map metadata derived from the installed game. They do not contain the original Paradox text files or map images. The bundle headers record the represented build, and both files are included in the source repository and compiled executable.

## Static population capacity

The committed dataset stores a population-capacity estimate derived only from immutable EU5 1.3.11 map data. It combines vegetation capacity and the location's map-coordinate distance from `equator_y`, then applies topography, climate, coastal, and five-tier river modifiers. Development, staffed buildings, location rank, laws, societal values, country modifiers, and other mutable campaign state are deliberately excluded. The total and the equator contribution are sortable columns, and the selected-location pane shows the full static breakdown.

## Filters

The application starts with every location visible, including oceans, lakes, and impassable locations. Impassability includes both `*_wasteland` topographies and EU5's explicit `impassable_mountains` map list. Categorical filters use bounded searchable checkbox lists; numeric ranges and exact RGB remain validated inputs. Filters combine with AND, and empty fields mean any value. Invalid numeric or RGB input is shown inline and does not change the active result set. The result table has sortable, resizable columns, a column-visibility menu covering every filter criterion and static capacity value, and draggable filter and detail pane boundaries.

## Scope and license

The importer reads only vanilla installation files. Mods, saves, ownership, population, and other campaign state are out of scope. Europa Universalis and Paradox Interactive are trademarks of their respective owners. This project is unofficial and is not affiliated with or endorsed by Paradox Interactive.

The source is available under MIT OR Apache-2.0. The UI uses Slint under the Slint Royalty-free License and includes Slint attribution in its About dialog.

Created by Young Hyun Chi. Project links: [GitHub](https://github.com/younghyun1), [cyhdev.com](https://cyhdev.com), and [LinkedIn](https://www.linkedin.com/in/young-hyun-chi-553431376/).
