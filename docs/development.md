# Development

The workspace uses Rust edition 2024 with MSRV 1.100 and the rolling nightly toolchain with `rust-src`, `rustfmt`, and `clippy`. Local builds target `x86_64-pc-windows-msvc`, use `target-cpu=native`, rebuild `core`, `alloc`, and `std`, and compile with the Rust 1.100 `immediate-abort` panic strategy. Cargo's abort-compatible test mode keeps the test harness aligned with the rebuilt standard library.

Run development verification with:

```text
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo doc --no-deps
cargo package --list
cargo publish --dry-run --locked
```

Do not run a release build as part of routine implementation. Tests use synthetic fixtures. The ignored real-install checks require `EU5_GAME_DIR`; the ignored filter timing test requires an existing data blob and reports repeated full-filter and name-search timings.

On build `24187685`, development-profile decompression and validation of both embedded bundles took 216 ms on the loader thread. Across 100 repeated runs, maximum full-filter latency was 7.29 ms, maximum name-search latency was 9.21 ms, and the slowest column sort scan was 5.36 ms. Interaction measurements remain below one 16.7 ms frame.

Search text is normalized once per record. Global search uses Rust's short-string substring search because the real dataset's individual name-and-ID haystacks are too small to amortize SIMD dispatch; searchable dropdowns use `memchr::memmem::Finder` across their longer option scan. Ascending and descending `LocationId` orders for every table column are built once on the loader thread. A filter or sort interaction therefore scans one already ordered bounded index and does not perform an interaction-time sort or grow a cache.
