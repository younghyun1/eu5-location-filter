# Development

The workspace uses Rust edition 2024 and the rolling nightly toolchain with `rust-src`, `rustfmt`, and `clippy`. Code remains compatible with Rust 1.92, the minimum supported version required by Slint 1.17.1. This native desktop application intentionally does not set a repository-wide musl target, rebuild the standard library, or replace the Windows allocator.

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
