---
# ish-353g
title: 'TUI: terminal-too-small error rendering'
status: todo
type: task
priority: high
tags:
- tui
created_at: 2026-04-25T03:20:55.760454Z
updated_at: 2026-04-25T03:21:17.799019Z
parent: ish-q6t1
blocked_by:
- ish-yfuo
- ish-wka9
---

## Goal
Render a clear centered error when the terminal is smaller than 80×20
instead of a broken layout.

## Scope
- In `update`, on `Msg::Resize(w, h)`, set `model.term_too_small = w < 80 || h < 20`.
- In `view::draw`, if `model.term_too_small` is true, draw only a
  centered message: `"Terminal too small (minimum 80×20)"`. Skip
  rendering of all other screens and the status line/footer.
- All key handling continues; the user can still `Ctrl-c` out.

## Files
- `src/tui/update.rs`, `src/tui/view/mod.rs`.

## Verification
- `mise run ci` passes.
- Manual smoke: shrink the terminal below 80×20 — the centered message
  appears; resize larger — the board returns.
- Unit test in `update`: a `Resize(40, 10)` followed by `Resize(120, 30)`
  flips the flag both ways.
