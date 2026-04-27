---
# ish-icc4
title: 'TUI: status picker modal view'
status: completed
type: task
priority: high
tags:
- tui
created_at: 2026-04-25T03:20:55.678563Z
updated_at: 2026-04-25T04:42:47.634656Z
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


## Implementation notes
- Added `src/tui/view/status_picker.rs` with a centered "Set status" modal that renders the full status list, applies shared status colors from `tui::theme`, and highlights the selected row for the existing picker navigation flow.
- Updated `src/tui/view.rs` so `Screen::StatusPicker(...)` now layers on top of the underlying `Screen::IssueDetail(...)` view instead of falling back to the generic placeholder, preserving the detail screen behind the modal as called for in the PRD.
- Added focused status-picker view coverage for both the rendered option list contents and a full-screen `TestBackend` smoke test that exercises the stacked detail+modal rendering path.

## Validation
- `mise exec -- cargo test tui::view::status_picker -- --nocapture`
- `mise run ci`
- `mise exec -- ish check`

## Follow-up notes
- The current overlay logic in `src/tui/view.rs` special-cases the detail+status-picker stack, which is enough for the current TUI design. If later work adds more modal screen types, consider generalizing this into an explicit layered-screen renderer instead of growing one-off match arms.
