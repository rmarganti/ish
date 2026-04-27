---
# ish-yfuo
title: 'TUI: implement pure update function'
status: completed
type: task
priority: high
tags:
- tui
created_at: 2026-04-25T03:20:55.535076Z
updated_at: 2026-04-25T03:59:00.997096Z
parent: ish-q6t1
blocked_by:
- ish-8dtp
---

## Goal
Implement the pure `update(Model, Msg) -> (Model, Vec<Effect>)` function
covering all per-screen logic.

## Scope
### `src/tui/update.rs`
- Top-level dispatch on `model.screens.last()`; falls back to global keys
  (`Quit`, `Tick`, `Resize`, `IssuesLoaded`, `DismissStatusLine`).
- One sub-function per screen kind: `update_board`, `update_detail`,
  `update_picker`, `update_create_form`, `update_help`.
- Board update implements:
  - No-wrap navigation across 4 columns.
  - Per-column cursor memory (cursor stays put when leaving and returning).
  - Empty columns are navigable but inert (no card selected).
  - `g`/`G`, `Ctrl-d`/`Ctrl-u` semantics.
  - Vertical scroll offset recomputed via cursor-in-view rule (assume a
    visible-row count carried in `BoardState` or computed from `term size`
    cached in model).
  - `r` emits `Effect::LoadIssues`.
  - `enter`/`space` pushes `IssueDetail` for the focused card.
  - `c` pushes `CreateForm`.
  - `?` pushes `Help`.
- Detail update implements body scroll, `e` → `Effect::OpenEditorForIssue`,
  `s` → push `StatusPicker` seeded with current status, `q`/`Esc` pops.
- StatusPicker update implements list nav, `enter` → `Effect::SaveIssue`
  with the issue's known ETag, then pops itself optimistically;
  `Esc`/`q` pops without saving.
- CreateForm update: tab/shift-tab/Ctrl-n/Ctrl-p between fields, type and
  priority cycling, character input on string fields, `Ctrl-s`/`enter`
  on submit field → `Effect::CreateIssue { open_in_editor: false }`,
  `Ctrl-e` → `... { open_in_editor: true }`, `Esc` cancels with confirm
  prompt if any field non-empty.
- Status-line lifecycle:
  - Set timestamp when status set; `Tick` clears info/success after ~3s.
  - Errors persist until `DismissStatusLine` or replaced after a 1s
    "stickiness" window.
- `IssuesLoaded(Ok)` replaces the cache and updates etags. Per-column
  cursors clamp to the new bucket sizes (do not move otherwise).
- `SaveFailed(Conflict)` sets a sticky error: `"ish-XXXX changed externally
  — press r to reload"`.

## Files
- `src/tui/update.rs`.

## Verification
- `mise run ci` passes.
- Unit tests live in the unit-test ish (depends on this); minimum here is
  that everything compiles and a single smoke test runs the empty board
  through a `Tick` without panicking.


## Implementation notes
- Replaced the placeholder `src/tui/update.rs` no-op with the first real pure update dispatcher, including per-screen handlers for board, detail, picker, create-form, and help screens.
- Added board navigation/state helpers for no-wrap horizontal movement, per-column cursor memory, vertical cursor clamping, and cursor-in-view offset maintenance over the four fixed kanban columns.
- Wired global message handling for issue reloads, resize-to-small-terminal state, save/editor status messages, and timed status-line expiration/error stickiness.
- Added create-form submission/cancel behavior and status-picker save behavior that emit the expected TUI effects without touching runtime or store code.
- Added a minimal `Tick` smoke test so the pure update path is exercised before the dedicated update-test ish lands.

## Validation
- `mise exec -- cargo test tui::update -- --nocapture`
- `mise exec -- cargo test`
- `mise run ci`
