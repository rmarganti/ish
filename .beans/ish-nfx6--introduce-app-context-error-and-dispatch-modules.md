---
# ish-nfx6
title: Introduce app context, error, and dispatch modules
status: completed
type: task
priority: high
created_at: 2026-04-17T19:01:25Z
updated_at: 2026-04-17T19:12:23Z
parent: ish-hqty
---

## Goal

Create an `src/app/` layer that owns shared application plumbing: loading project context, mapping errors, and dispatching parsed commands.

## Context

Shared logic currently mixed into `src/main.rs`:
- `RunOutcome`
- `AppError`
- `run(cli)` dispatch
- store/config discovery and loading (`load_store_from_current_dir`)
- error mapping helpers (`store_app_error`, `store_open_error`, `json_output_error`, `classify_app_error`)

These concerns are used by multiple commands and should not remain duplicated or buried inside the entrypoint file.

## Scope

- Create `src/app/mod.rs`, `src/app/context.rs`, and `src/app/error.rs`.
- Move shared context loading into an app-level helper or `AppContext` struct.
- Move app-level error translation into `src/app/error.rs`.
- Move top-level command dispatch into `src/app/mod.rs`.

## Success Criteria

- [x] Shared store/config bootstrap logic no longer lives in `src/main.rs`.
- [x] Error classification/mapping is centralized in `src/app/error.rs`.
- [x] Command dispatch is readable and no longer buried among command implementations.

## Verification

- [x] `cargo test`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy -- -D warnings`
- [x] At least one representative command (`list`, `show`, or `create`) still works through the new app layer.

## Summary of Changes

- Added `src/app/mod.rs`, `src/app/context.rs`, and `src/app/error.rs` to own top-level dispatch, shared context loading, and app error translation.
- Updated `src/main.rs` to delegate command routing to the new app layer while keeping command implementations intact.
- Added a regression test that exercises `run()` dispatching `list` through the new app layer.

## Notes for Future Workers

- `load_store_from_current_dir()` in `src/main.rs` is now only a thin compatibility wrapper over `AppContext::load()` for existing tests/helpers; shared bootstrap logic lives in `src/app/context.rs`.
- Top-level app error mapping now lives in `src/app/error.rs`, so new entrypoint-facing error conversions should be added there rather than back in `src/main.rs`.
