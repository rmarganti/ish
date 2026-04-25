---
# ish-loy6
title: 'TUI: scaffolding, deps, and ish tui CLI entry'
status: todo
type: task
priority: high
tags:
- tui
created_at: 2026-04-25T03:20:55.487431Z
updated_at: 2026-04-25T03:20:55.487431Z
parent: ish-q6t1
---

## Goal
Lay the foundation for the TUI feature: add dependencies, create the
`src/tui/` module skeleton, and wire a no-op `ish tui` subcommand.

## Scope
- Add to `Cargo.toml`: `ratatui` and `crossterm` (latest compatible
  versions). Build via `mise exec -- cargo build` to populate `Cargo.lock`.
- Create `src/tui/mod.rs` exporting empty submodules: `model`, `msg`,
  `keymap`, `update`, `effect`, `runtime`, `view`, `theme`. Add
  `pub fn run(ctx: &AppContext) -> AppResult<()>` that returns `Ok(())`
  for now.
- Declare the new `tui` module from `src/main.rs` (or `src/lib.rs` if it
  exists — confirm by reading them).
- Create `src/commands/tui.rs` with a `Tui` clap subcommand (no flags) and
  register it in `src/commands/mod.rs` and the CLI dispatcher under
  `src/cli/`. Mirror an existing simple command (e.g. `commands/version.rs`)
  for style.
- The subcommand handler calls `tui::run(ctx)`.

## Files
- `Cargo.toml`
- `src/tui/mod.rs` (new) and submodule stubs
- `src/commands/tui.rs` (new)
- `src/commands/mod.rs`, `src/cli/...` — register the subcommand

## Verification
- `mise run ci` passes.
- `mise exec -- cargo run -- tui` exits 0 silently.
- `mise exec -- cargo run -- --help` lists `tui` as a subcommand.
