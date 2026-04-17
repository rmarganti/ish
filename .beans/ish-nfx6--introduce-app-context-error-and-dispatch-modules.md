---
# ish-nfx6
title: Introduce app context, error, and dispatch modules
status: todo
type: task
priority: high
created_at: 2026-04-17T19:01:25Z
updated_at: 2026-04-17T19:02:06Z
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

- [ ] Shared store/config bootstrap logic no longer lives in `src/main.rs`.
- [ ] Error classification/mapping is centralized in `src/app/error.rs`.
- [ ] Command dispatch is readable and no longer buried among command implementations.

## Verification

- [ ] `cargo test`
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy -- -D warnings`
- [ ] At least one representative command (`list`, `show`, or `create`) still works through the new app layer.
