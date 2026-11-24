# Changelog

## 0.1.0 - Unreleased

- Measure the optimized browser package at 9.4 MB raw and 5.5 MB gzip, against 14.3 MB and 8.3 MB for the previous interface on the same data.
- Ship the IBM Plex Sans and Mono faces unmodified and give every subset font its own family name, because the IBM Plex licence reserves the name for unmodified fonts.
- Replace the Slint interface with iced 0.14 on desktop and in the browser (wgpu with a WebGL 2 fallback). The result grid and the checklists are custom widgets that draw only visible rows. Slint and its royalty-free licence attribution are removed; the About dialog lists iced, IBM Plex and Noto licences.
- Redesign the interface after the cyhdev site: twelve host palette colours with a short palette transition, five accent presets on desktop, reduced motion, and keyboard operation of every control. iced 0.14 has no screen-reader tree, which is a documented limit.
- Add interface catalogs for the site's 27 locales and right-to-left layout for Arabic, Persian and Urdu. Only English and Korean are written directly; the other catalogs are drafts that need native review.
- Add game-text language packs for French, German, Spanish, Brazilian Portuguese, Polish, Russian, Turkish, Japanese, Korean and Simplified Chinese, with localized names, labels, search text and sort orders. Index schema 6 splits the language-independent index from a per-language section; symbol columns now sort by their displayed label instead of their identifier. Desktop embeds the packs; the browser fetches one on demand and rejects a pack from another build.
- Add IBM Plex Sans and Mono for Latin, Vietnamese and Cyrillic, and font packs for Korean, Japanese, Chinese, Arabic, Devanagari, Bengali and Telugu.
- Add a narrow layout with drawers and touch scrolling, which the browser build uses below 768 pixels.
- Replace the string theme messages with a versioned host protocol that carries the palette, locale, direction, reduced motion and a bounded view-state string for shareable links, and add a DOM search field in the browser so Korean, Japanese and Chinese input and paste work.
- Add typed import progress stages and the `--languages` and `--lang-dir` import options, and extend `diff` and `reindex` to language packs.
- Bundle adaptive facet bitmaps, numeric range indexes, substring postings, and inverse sort ranks; add offline reindexing and scan-equivalence timing tests.
- Add the initial EU5 location importer, versioned data blob, filters, command-line interface, and desktop UI.
- Embed level-22 location and filter-index bundles for EU5 1.4.8 build 26547371, replacing the 1.3.11 data: 5,739 locations change climate under the reworked climate set, 4,582 static capacities follow, 261 names and 126 raw materials change, and horses count as food-producing.
- Accept scalar attributes inside area and province blocks of `definitions.txt`, which EU5 1.4 introduces.
- Add a `diff` subcommand that compares two location bundles field by field.
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
- Align the global search field with the result pane and make the chart boundary continuous through unused table space and the scrollbar.
