---
# ish-hqty
title: Refactor CLI/app module layout and shrink main.rs
status: completed
type: epic
priority: high
created_at: 2026-04-17T19:01:08Z
updated_at: 2026-04-17T19:37:50Z
---

## Goal

Refactor the current 3.5k-line `src/main.rs` into a clearer feature-oriented module hierarchy so the entrypoint is small, command logic is localized, and shared app plumbing is centralized.

## Context

Current codebase observations:
- `src/main.rs` is ~3562 lines and mixes CLI type definitions, top-level dispatch, command handlers, command-specific helpers, shared I/O helpers, output shaping, error translation, and a very large test module.
- Existing modules already provide useful domain boundaries: `config`, `core`, `model`, `output`, `cli`, and `roadmap`.
- The biggest missing layer is a dedicated `commands/` area plus a small `app/` layer for shared bootstrap/context/error concerns.

Target direction:
- Keep `src/main.rs` tiny: parse args, call app runner, print output, exit.
- Introduce `src/app/` for context loading, dispatch, and app-level errors.
- Move clap structs into `src/cli/args.rs`.
- Introduce `src/commands/` with one module per command.
- Split especially heavy command implementations (`list`, `check`) into submodules.
- Defer deeper `roadmap` / `store` internal splits until after the command extraction lands cleanly.

## Success Criteria

- [x] `src/main.rs` only contains bootstrap/entrypoint concerns.
- [x] Command logic lives under `src/commands/` instead of `src/main.rs`.
- [x] Shared context/error handling lives under `src/app/`.
- [x] CLI arg definitions live under `src/cli/args.rs`.
- [x] The refactor preserves behavior and passes the full feedback loop.

## Verification

- [x] `cargo test`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy -- -D warnings`

## Summary of Changes

- Extracted clap argument definitions into `src/cli/args.rs` and moved dispatch/bootstrap concerns into `src/app/`.
- Moved command implementations out of `src/main.rs` into `src/commands/`, with focused submodules for the heaviest `list` and `check` command logic.
- Finished the refactor by shrinking `src/main.rs` to a small entrypoint and relocating the old centralized tests into module-local `#[cfg(test)]` suites.
- Added `src/test_support.rs` so shared test scaffolding stays reusable without recreating a new monolithic test file.
- Verified the refactor with the full feedback loop (`cargo test`, `cargo fmt --all -- --check`, `cargo clippy -- -D warnings`).
