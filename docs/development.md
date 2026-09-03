# Development

The workspace uses Rust edition 2024 with MSRV 1.100 and the rolling nightly toolchain. Cargo uses the host target by default, so normal builds work directly on Windows and Linux; release automation selects each supported target explicitly. x86-64 builds use `target-cpu=x86-64-v3`, while ARM64 builds use the generic AArch64 ARMv8-A baseline. Optimized builds use the portable `abort` panic strategy without rebuilding the standard library.

Run development verification with:

```text
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo doc --no-deps
cargo package --list
cargo publish --dry-run --locked
```

The browser target follows Slint's WebAssembly build model and renders into `web/index.html`'s `canvas#canvas`. Build an unoptimized development package with:

```text
wasm-pack build --dev --target web --no-pack --no-typescript -- --locked --no-default-features --features web
```

This writes ignored JavaScript and WebAssembly output to `pkg/`. Serve the repository root over HTTP and open `/web/`; browser ES module loading does not work from a `file://` URL. Production builds use the release profile and run `wasm-opt -Oz` through the package metadata.

Do not run a release build as part of routine implementation. Tests use synthetic fixtures. The ignored real-install checks require `EU5_GAME_DIR`; the ignored filter timing test requires an existing data blob and reports repeated full-filter and name-search timings.

## GitHub automation

`CI` runs formatting, Clippy, tests, documentation, and source-package verification on Windows. A separate matrix compile-checks native x86-64 and ARM64 targets on Windows, Linux, and macOS. Linux jobs install the desktop libraries required by Slint's Winit backend. A WASM job runs target-specific Clippy and a development `wasm-pack` smoke build, then checks the generated JavaScript and WASM magic. Workflow actions are pinned to immutable commit hashes.

The tagged-release workflow is dormant until a `v*` tag is pushed. It rejects a tag that does not equal `v` plus the Cargo package version, whose commit is not on `main`, or whose required signing secrets are absent. Successful tags build six native archives; apply Authenticode and Apple Developer ID signatures; notarize and staple macOS applications; generate SHA-256 checksums and GPG signatures; attach provenance attestations; and create a GitHub Release from the existing tag. Configure the credentials in [release signing](signing.md), then prepare a release by updating `Cargo.toml` and `CHANGELOG.md`, committing those changes, creating an annotated matching tag, and pushing that tag. Crates.io publication remains a separate explicit action.

On build `24187685`, development-profile decompression and validation of both embedded bundles took 258 ms on the loader thread. Across 100 repeated runs, maximum full-filter latency was 12.83 ms, maximum name-search latency was 11.22 ms, and the slowest column sort scan was 9.66 ms. Interaction measurements remain below one 16.7 ms frame.

Search text is normalized once per record. Global search uses Rust's short-string substring search because the real dataset's individual name-and-ID haystacks are too small to amortize SIMD dispatch; searchable checkbox lists use `memchr::memmem::Finder` across their longer option scan. Ascending and descending `LocationId` orders for every table column are generated with the committed asset bundle. A filter or sort interaction therefore scans one already ordered bounded index and does not perform an interaction-time sort or grow a cache.
