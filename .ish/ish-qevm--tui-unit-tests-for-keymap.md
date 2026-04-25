---
# ish-qevm
title: 'TUI: unit tests for keymap'
status: todo
type: task
priority: high
tags:
- tui
created_at: 2026-04-25T03:20:55.823444Z
updated_at: 2026-04-25T03:21:17.817252Z
parent: ish-q6t1
blocked_by:
- ish-jrw0
- ish-fww3
---

## Goal
Per-screen unit tests asserting `keymap::map_key` returns the right
`Msg` for every documented binding, and that bindings do not leak across
screens.

## Scope
- `#[cfg(test)] mod tests` in `src/tui/keymap.rs`.
- One submodule per screen (`board`, `detail`, `picker`, `create_form`,
  `help`).
- For each screen, assert each binding maps to its `Msg`.
- A leak test: pick 3 bindings unique to one screen and assert they
  return `None` on the other screens.
- Ctrl-c → Quit on every screen.

## Files
- `src/tui/keymap.rs`.

## Verification
- `mise run ci` passes.
