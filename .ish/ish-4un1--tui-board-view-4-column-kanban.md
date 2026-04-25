---
# ish-4un1
title: 'TUI: board view (4-column kanban)'
status: todo
type: task
priority: high
tags:
- tui
created_at: 2026-04-25T03:20:55.637162Z
updated_at: 2026-04-25T03:21:17.753324Z
parent: ish-q6t1
blocked_by:
- ish-8dtp
- ish-5017
---

## Goal
Render the kanban board: 4 equal-width columns with header, scrolling
card list, fixed-height cards.

## Scope
### `src/tui/view/board.rs`
- `pub fn draw(frame: &mut Frame, area: Rect, model: &Model, state: &BoardState)`.
- 4 equal-width columns (use `Layout::horizontal` with `Constraint::Ratio(1,4)`).
- Column header shows status name + count, highlighted when active.
- Cards: fixed 2-line height (plus a 1-line border row if bordered).
  - Line 1: title, truncated with `…` to fit width.
  - Line 2: `id  ! priority  ⊙ type  #firstTag`, with type/priority/tag
    colored from `theme`.
  - Selected card has a highlighted border/background.
- Empty columns render a dim `(empty)` placeholder.
- Vertical scroll uses `state.column_offsets[col]` so the cursor is
  always in view (slice the bucket before drawing).
- Sort order matches `Model::bucket_for_status` (priority desc, updated_at desc).

## Files
- `src/tui/view/mod.rs` (top-level dispatcher; create if missing)
- `src/tui/view/board.rs`

## Verification
- `mise run ci` passes.
- Manual smoke: `cargo run -- tui` shows current workspace ishes bucketed
  correctly; navigation moves the highlight; empty columns show the
  placeholder.
