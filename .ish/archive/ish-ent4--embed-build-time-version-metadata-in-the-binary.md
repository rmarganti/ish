---
# ish-ent4
title: Embed build-time version metadata in the binary
status: completed
type: task
priority: normal
created_at: 2026-04-20T03:01:07.971650Z
updated_at: 2026-04-20T03:05:07.585822Z
parent: ish-gv4k
blocking:
- ish-dki9
---

## Context
`src/commands/version.rs` currently prints `env!("CARGO_PKG_VERSION")`, which only reflects the crate version. The release workflow needs the binary to report the release tag/version that was dispatched, so the version has to be injected at build time in a way that works for release builds and still has a sensible fallback for local development.

## Work
- Choose the idiomatic Rust mechanism for embedding release metadata (`build.rs` with `cargo:rustc-env`, or an equivalent approach).
- Make sure `ish version` and Clap's built-in `--version` both reflect the embedded release version.
- Preserve a sensible dev/build fallback so local `cargo test` and normal CI still behave predictably.

## Verification
- Update or add tests around version output, including JSON mode.
- `mise run ci` passes.
- A release build can be produced with an injected version and the binary prints that exact value.


## Implementation notes
- Added `build.rs` to publish a compile-time `ISH_BUILD_VERSION` via `cargo:rustc-env`, falling back to `CARGO_PKG_VERSION` when no release version is injected.
- Added `src/version.rs` as the single source of truth for the resolved build version so both `ish version` and Clap's built-in `--version` read the same value.
- Updated CLI/tests to assert the built-in Clap version banner matches the injected build version and kept JSON `ish version` coverage through the app layer.

## Validation
- `mise exec -- cargo test`
- `ISH_BUILD_VERSION=9.9.9 mise exec -- cargo run -- version`
- `ISH_BUILD_VERSION=9.9.9 mise exec -- cargo run -- --version`
- `mise run ci`
