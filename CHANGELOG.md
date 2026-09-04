# Changelog

## 0.1.0 - Unreleased

- Add the initial EU5 location importer, versioned data blob, filters, command-line interface, and desktop UI.
- Embed level-22 location and filter-index bundles for EU5 1.3.11 build 24187685.
- Add bounded searchable checkbox filters, configurable resizable columns, pane splitters, ASCII-folded search, and resolved localization references.
- Precompute five-tier gameplay river bonuses and static population capacity, including the location's closeness-to-equator contribution.
- Include EU5's explicit impassable-location list, exclude those locations from static capacity, and correct filter and column resizing geometry.
- Add Young Hyun Chi's LinkedIn profile to the About dialog.
- Move the global clear action into the Visibility header and make the full filter rail collapsible.
- Add per-filter folding and balanced padding around result-column separators.
- Elide column labels and cell text when a user narrows a column while keeping the active sort arrow visible.
- Target x86-64-v3 and baseline ARMv8-A for reproducible release binaries.
- Add six-platform CI checks and a signed, notarized, version-validated tagged GitHub Release workflow without creating a release.
- Make every checkbox filter non-exclusive within its field, rename Kind to Type, and default Type to Land.
- Let the Columns button close its own selector and make the detail pane independently collapsible.
- Use neutral modal scrims in both themes without translucent-color interpolation artifacts.
- Make Raw material a default column with plain labels, and correct silver highlighting.
- Add bounded pure Rust browser decompression and choose measured-fastest WebAssembly optimization with Rust O3, fat LTO, SIMD128, and Binaryen Oz.
