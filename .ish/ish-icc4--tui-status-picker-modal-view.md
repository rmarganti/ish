---
# ish-icc4
title: 'TUI: status picker modal view'
status: todo
type: task
priority: high
tags:
- tui
created_at: 2026-04-25T03:20:55.678563Z
updated_at: 2026-04-25T03:21:17.773003Z
parent: ish-q6t1
blocked_by:
- ish-8dtp
- ish-5017
- ish-nxj6
---

## Goal
Render the status-picker modal pushed on top of the issue detail view.

## Scope
### `src/tui/view/status_picker.rs`
- `pub fn draw(frame: &mut Frame, area: Rect, model: &Model, state: &PickerState)`.
- Centered modal (~40 cols × 8 rows). Use a ratatui `Block` with title
  "Set status" and a list of options.
- Highlight the selected row using theme colors.
- The underlying detail screen continues to render behind the modal
  (draw it first, then the modal — the dispatcher in `view/mod.rs`
  should know to layer modals).

## Files
- `src/tui/view/status_picker.rs`
- Update `src/tui/view/mod.rs` so the dispatcher walks the screen stack
  and renders the underlying screen plus the modal overlay.

## Verification
- `mise run ci` passes.
- Manual smoke: from detail view, `s` opens the modal; `j/k` moves
  selection; `enter` saves and pops back; `Esc`/`q` cancels.
