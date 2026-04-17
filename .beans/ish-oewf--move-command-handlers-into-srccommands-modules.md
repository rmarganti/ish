---
# ish-oewf
title: Move command handlers into src/commands/ modules
status: todo
type: task
priority: high
created_at: 2026-04-17T19:01:36Z
updated_at: 2026-04-17T19:02:06Z
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

- [ ] `src/main.rs` no longer contains concrete command handler implementations.
- [ ] `src/commands/` becomes the obvious home for command behavior.
- [ ] Public interfaces between commands and app dispatch are simple and consistent.

## Verification

- [ ] `cargo test`
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy -- -D warnings`
- [ ] Smoke-check a representative set of commands: `ish list`, `ish show`, `ish create`, `ish check`, `ish roadmap`.
