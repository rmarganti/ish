---
# ish-hqty
title: Refactor CLI/app module layout and shrink main.rs
status: todo
type: epic
priority: high
created_at: 2026-04-17T19:01:08Z
updated_at: 2026-04-17T19:22:40Z
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

- [ ] `src/main.rs` only contains bootstrap/entrypoint concerns.
- [ ] Command logic lives under `src/commands/` instead of `src/main.rs`.
- [ ] Shared context/error handling lives under `src/app/`.
- [ ] CLI arg definitions live under `src/cli/args.rs`.
- [ ] The refactor preserves behavior and passes the full feedback loop.

## Verification

- [ ] `cargo test`
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy -- -D warnings`
