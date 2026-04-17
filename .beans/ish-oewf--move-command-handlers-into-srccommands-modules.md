---
# ish-oewf
title: Move command handlers into src/commands/ modules
status: completed
type: task
priority: high
created_at: 2026-04-17T19:01:36Z
updated_at: 2026-04-17T19:22:25Z
parent: ish-hqty
blocked_by:
    - ish-t1q0
    - ish-nfx6
---

## Goal

Extract command implementations from `src/main.rs` into a dedicated `src/commands/` hierarchy with one module per command.

## Context

Command entrypoints currently living in `src/main.rs`:
- `init_command`
- `create_command`
- `list_command`
- `show_command`
- `update_command`
- `delete_command` / `delete_command_with_io`
- `archive_command`
- `check_command`
- `prime_command`
- `roadmap_command`
- version output handling

This is the largest structural extraction and should happen after the CLI arg and app-plumbing moves make the remaining file easier to work in.

## Scope

- Create `src/commands/mod.rs`.
- Create one file per command (`init.rs`, `create.rs`, `list.rs`, `show.rs`, `update.rs`, `delete.rs`, `archive.rs`, `check.rs`, `roadmap.rs`, `prime.rs`, `version.rs`).
- Move command-specific helpers alongside the command that owns them.
- Keep shared helpers in `app/` or existing generic modules only when they are truly cross-command.

## Success Criteria

- [x] `src/main.rs` no longer contains concrete command handler implementations.
- [x] `src/commands/` becomes the obvious home for command behavior.
- [x] Public interfaces between commands and app dispatch are simple and consistent.

## Verification

- [x] `cargo test`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy -- -D warnings`
- [x] Smoke-check a representative set of commands: `ish list`, `ish show`, `ish create`, `ish check`, `ish roadmap`.

## Summary of Changes

- Added `src/commands/` with dedicated modules for archive, check, create, delete, init, list, prime, roadmap, show, update, and version behavior.
- Updated `src/app/mod.rs` to dispatch directly through the new command layer instead of reaching into `main.rs`.
- Reduced `src/main.rs` to module wiring, entrypoint logic, and existing tests, with test-only re-exports/helpers kept in place so behavior coverage stayed intact during the extraction.

## Notes for Future Workers

- `src/main.rs` is still large because the command integration tests remain there; the next slimming pass can move or redistribute those tests after the remaining refactor beans land.
- `list` and `check` each now live in their own command modules, making the planned submodule split in `ish-ywbj` a localized follow-up rather than another `main.rs` extraction.
