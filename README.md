# EU5 Location Filter

EU5 Location Filter is an unofficial desktop browser for the static map locations in a vanilla Europa Universalis V installation. It imports location attributes, hierarchy, colors, ports, and rivers into a local compressed data file, then provides fast compound filtering without reading the game again.

## Install and run

Install from a source checkout with Rust 1.92 or newer:

```text
cargo install --path . --locked
eu5-location-filter
```

On Windows, the first run discovers Steam through the registry and reads app ID `3450310`. On other systems, or for a nonstandard installation, pass `--game-dir PATH`. The default data file is `./eu5-locations.bitcode.zst`. A normal start imports only when that file is absent. Use the Rebuild action or `eu5-location-filter import --force` to replace it explicitly.

The data file contains names and static map metadata derived from the user's installed game. It can be copyrighted game data and can disclose the installed build. Do not redistribute it. Generated blobs and original Paradox files are excluded from this source package.

## Filters

The application starts with every location visible, including oceans, lakes, and impassable wastelands. Filters combine with AND; multiple values in one categorical filter combine with OR. Empty fields mean any value. Invalid numeric or RGB input is shown inline and does not change the active result set.

## Scope and license

The importer reads only vanilla installation files. Mods, saves, ownership, population, and other campaign state are out of scope. Europa Universalis and Paradox Interactive are trademarks of their respective owners. This project is unofficial and is not affiliated with or endorsed by Paradox Interactive.

The source is available under MIT OR Apache-2.0. The UI uses Slint under the Slint Royalty-free License and includes Slint attribution in its About dialog.
