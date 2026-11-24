# Development

The workspace uses Rust edition 2024 with MSRV 1.100 and the rolling nightly toolchain. Cargo uses the host target by default, so normal builds work directly on Windows and Linux; release automation selects each supported target explicitly. x86-64 builds use `target-cpu=x86-64-v3`, while ARM64 builds use the generic AArch64 ARMv8-A baseline. Optimized builds use the portable `abort` panic strategy without rebuilding the standard library.

The interface is iced 0.14.0, pinned exactly, with the `advanced`, `advanced-shaping`, `crisp`, `web-colors` and `wgpu` features everywhere, `thread-pool`, `tiny-skia`, `wayland` and `x11` on native targets, and `webgl` on wasm32. The `desktop` feature (default) builds the window, importer and command line; the `web` feature builds the browser package on wasm32. `cargo check --no-default-features` builds only the data core, without iced. On Linux, winit loads the X11, Wayland and xkbcommon libraries at run time, so a window needs them installed but compiling does not need their headers; the wgpu renderer needs a Vulkan or OpenGL driver, and when no GPU adapter exists the software renderer takes over. The interface code lives in `src/app/`; `docs/design/fe/layout.md` and `docs/design/fe/host-protocol.md` define the layout and the frame protocol.

Run development verification with:

```text
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --target wasm32-unknown-unknown --no-default-features --features web -- -D warnings
cargo test --locked --all-targets
cargo doc --locked --no-deps
cargo package --list
cargo publish --dry-run --locked
```

The wasm32 Clippy and build commands need `RUSTFLAGS` set to the wasm32 flags of `.cargo/config.toml` plus `--cfg=web_sys_unstable_apis`, which wgpu's WebGPU bindings require; `RUSTFLAGS` replaces the configured flags, so the target features must be repeated (see `WASM_RUSTFLAGS` in `.github/workflows/ci.yml`). Interface tests drive the application with iced's headless `Simulator` (`iced_test`, native targets only), and the interface text catalogs, the font coverage and the language packs have their own parity and validation tests. Tests that need the unavailable game installation, generated fonts or optimized builds are ignored by default.

The browser target renders with wgpu into a canvas that iced creates in place of the `canvas` placeholder of `web/index.html`, which also hosts the frame protocol and the DOM search field. Build an unoptimized development package with:

```text
wasm-pack build --dev --target web --out-dir web/pkg --no-pack --no-typescript -- --locked --no-default-features --features web
```

This writes ignored JavaScript and WebAssembly output to `web/pkg/`. Serve the repository root over HTTP and open `/web/`; browser ES module loading does not work from a `file://` URL. Opened directly, the page runs standalone with built-in colours and English text. Language packs and font packs are fetched from `lang/` and `fonts/` next to `index.html`, so serve copies of `assets/lang/*.bitcode.zst` and `assets/fonts/*.ttf` there to test them; without them the application keeps English game text and the embedded fonts. The browser needs WebGPU or WebGL 2. Production builds use Rust `-O3`, one codegen unit, fat LTO, SIMD128, and standardized post-MVP WebAssembly instructions. Binaryen then runs `-Oz`. The following comparison was measured on the previous interface with the same data scans: an Edge comparison against `-O3` measured 567.6 ms versus 577.1 ms median for 500 real indexed filter scans with identical checksums. The `-Oz` artifact was 204,764 bytes smaller raw and 8,212 bytes smaller after fast gzip, so it was both faster and smaller in the measured workload. The optimized size, gzip size and startup time of the iced build are recorded here after the integration run, next to the numbers of the previous interface; this document states no figure for them yet. Browser decoding uses bounded pure Rust zstd with the committed level-22 frame's 128 MiB window, avoiding a C cross-compiler while retaining the 128 MiB decompressed-output limit.

Do not run a release build as part of routine implementation. Parser tests use synthetic fixtures. The ignored imported-data check reads `EU5_DATA_FILE`, defaulting to `eu5-locations.bitcode.zst`; it does not access Steam. Timing tests read the committed bundles. Fonts and language packs are generated, not committed by hand: the import writes the packs, and a font-pack subcommand builds the font files from local source fonts and records their source version and SHA-256. Licence texts for the shipped fonts and iced are in `assets/licenses/`. The non-default `web-benchmark` feature exports bounded real-data queries used for WebAssembly optimizer comparisons; it is absent from normal browser packages.

## GitHub automation

`CI` runs formatting, Clippy, tests, documentation, and source-package verification on Windows. A separate matrix compile-checks native x86-64 and ARM64 targets on Windows, Linux, and macOS. Linux jobs install the xkbcommon, Wayland and X11 development packages for winit. The quality job also checks the data core with `--no-default-features`. A WASM job runs target-specific Clippy and an optimized `wasm-pack` smoke build with the wasm32 `RUSTFLAGS` above, including Binaryen validation, then checks the generated JavaScript and WASM magic. Workflow actions are pinned to immutable commit hashes.

The tagged-release workflow is dormant until a `v*` tag is pushed. It rejects a tag that does not equal `v` plus the Cargo package version, whose commit is not on `main`, or whose required signing secrets are absent. Successful tags build six native archives; apply Authenticode and Apple Developer ID signatures; notarize and staple macOS applications; generate SHA-256 checksums and GPG signatures; attach provenance attestations; and create a GitHub Release from the existing tag. Configure the credentials in [release signing](signing.md), then prepare a release by updating `Cargo.toml` and `CHANGELOG.md`, committing those changes, creating an annotated matching tag, and pushing that tag. Crates.io publication remains a separate explicit action.

## Query indexes and local timings

Regenerate only the query bundle with `cargo run --locked -- reindex --force`. This reads the committed location blob, not EU5 files. Run `cargo test --lib --locked filter::timings:: -- --ignored --nocapture --test-threads=1` for scan-equivalence checks, interleaved timings, output-order crossover tests, and deterministic byte-for-byte bundle regeneration. The previous record predicate is compiled only into unit tests; production queries do not retain a fallback record scan or a growing cache.

Categorical selections use compact sparse postings or dense bitmaps, with OR inside each field and AND between fields. Numeric range boundaries use binary search, followed by materializing the matching IDs. Exact RGB uses the existing bounded color lookup. Search normalizes input once, then verifies exact substrings against the rarest bundled trigram posting or the current candidate mask, whichever has fewer IDs. One/two-byte searches verify the current candidates. Normalization retains alphanumeric characters from every script while folding common Latin diacritics and ignoring punctuation and Unicode whitespace; the separate sort key preserves punctuation and spacing.

Every column has precomputed ascending and descending orders and inverse integer ranks. Full results copy their existing order. Sparse results sort only matching IDs by rank when `k * (floor(log2(k)) + 1) < n / 2`; dense results scan the existing order with constant-time bitmap membership. This conservative threshold was checked against direct order scanning for cardinalities from 1 through 28,573, including a 1,024-row rank sort. The complete query is not O(1) or O(log n): bitmap operations cost O(n/64), numeric materialization costs O(r), candidate verification depends on query length and candidate count, and emitting k IDs costs at least O(k).

On 2026-11-24, Rust 1.102 nightly, Linux x86-64-v3, and EU5 1.4.8 build `26547371`, the unoptimized development test measured the following medians after 10 warm-ups and 101 alternating indexed/scan samples. Each query's IDs were also compared with the old scan across all 25 columns in both directions. These timings are local diagnostic measurements, not native release or browser results. The index schema 6 split and the per-language sections change what is built per language; the same table for a language pack, the sizes of the packs and the interface frame timings (scroll, resize and typing, native and browser) are recorded by the integration run, and this document states none until then.

| Query | Rows | Previous scan | Indexed | Speedup |
| --- | ---: | ---: | ---: | ---: |
| All locations | 28,573 | 2.190 ms | 0.006 ms | 340x |
| Default Land | 21,041 | 4.151 ms | 0.471 ms | 8.8x |
| Land, coastal, food-producing | 2,695 | 6.060 ms | 0.363 ms | 17x |
| Harbor 0.5 through 0.75 | 415 | 1.837 ms | 0.094 ms | 19x |
| One-letter search `a` | 22,773 | 2.669 ms | 1.593 ms | 1.7x |
| Substring `stock` | 6 | 3.083 ms | 0.018 ms | 171x |
| First record's province | 7 | 3.483 ms | 0.013 ms | 276x |
| Exact RGB | 1 | 1.358 ms | 0.010 ms | 136x |

The query bundle is 2,226,765 compressed bytes; its bitcode payload is 7,643,748 bytes. The location bundle is 718,624 bytes. Decompression and validation of both bundles plus engine restoration took 153 ms in this run. Query scratch space is two 447-word bitmaps for 28,573 records, plus the result ID vector. All stored indexes and lookup tables are bounded by the immutable dataset. On the previous interface, all 74 tests passed locally, including the normally ignored timings, imported-blob checks, and deterministic index regeneration.
