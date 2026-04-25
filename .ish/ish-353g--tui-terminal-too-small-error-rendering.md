---
# ish-353g
title: 'TUI: terminal-too-small error rendering'
status: completed
type: task
priority: high
tags:
- tui
created_at: 2026-04-25T03:20:55.760454Z
updated_at: 2026-04-25T08:31:11.480508Z
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


## Implementation notes
- Updated `src/tui/view.rs` so `view::draw(...)` now short-circuits to a centered `"Terminal too small (minimum 80×20)"` message whenever `model.term_too_small` is set, skipping the normal board/detail rendering plus the shared status-line/footer chrome.
- Added a focused view test proving the too-small renderer suppresses normal chrome even when a status line would otherwise be present.
- Added the requested pure-update regression test in `src/tui/update.rs` confirming `Msg::Resize(40, 10)` sets `term_too_small = true` and `Msg::Resize(120, 30)` clears it again.

## Validation
- `mise exec -- cargo test terminal_too_small -- --nocapture`
- `mise run ci`
- `mise exec -- ish check`

## Follow-up notes
- `model.term_too_small` was already being maintained in `src/tui/update.rs`; this task finished the rendering side and pinned the resize behavior with an explicit unit test so future layout refactors do not silently regress the minimum-terminal contract.
