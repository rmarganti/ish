---
# ish-qevm
title: 'TUI: unit tests for keymap'
status: completed
type: task
priority: high
tags:
- tui
created_at: 2026-04-25T03:20:55.823444Z
updated_at: 2026-04-25T08:33:04.441024Z
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

## Implementation notes
- Added `#[cfg(test)]` coverage directly in `src/tui/keymap.rs` with one submodule each for board, detail, status-picker, create-form, help, and cross-screen leak behavior.
- The tests now pin every currently documented binding to its expected `Msg`, including the global `Ctrl-c` quit path and the cross-screen `?` help toggle where applicable.
- Create-form coverage is split by focused field so selector-only bindings (`h`/`l`, arrows) and text-entry bindings are asserted against the same field-sensitive behavior that runtime/update rely on.

## Validation
- `mise exec -- cargo test tui::keymap -- --nocapture`
- `mise exec -- ish check`
- `mise run ci`

## Follow-up notes
- The leak test intentionally uses create-form-only bindings against board/detail/picker because the help overlay is designed to consume any key as `PopScreen`; if the help contract changes later, revisit that expectation explicitly rather than broadening the leak assertions accidentally.
