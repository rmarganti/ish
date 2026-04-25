---
# ish-nxj6
title: 'TUI: issue detail view (metadata + scrollable body)'
status: todo
type: task
priority: high
tags:
- tui
created_at: 2026-04-25T03:20:55.658047Z
updated_at: 2026-04-25T03:21:17.761153Z
parent: ish-q6t1
blocked_by:
- ish-8dtp
- ish-5017
---

## Goal
Render the issue detail screen: metadata block + scrollable markdown body.

## Scope
### `src/tui/view/issue_detail.rs`
- `pub fn draw(frame: &mut Frame, area: Rect, model: &Model, state: &DetailState)`.
- Top metadata block: title, id, type (colored), status (colored),
  priority (colored), tags, parent, blocking, blocked_by, updated_at.
- Body: render markdown. Acceptable v1: pass through plain text or use
  `termimad`-rendered text reflowed into ratatui Paragraph; if a clean
  ratatui markdown widget is not available, fall back to monospace text
  with simple heading/code-block styling.
- Scroll respects `state.scroll`.
- Footer keybind hints: `e edit  s status  q back`.

## Files
- `src/tui/view/issue_detail.rs`
- Register the screen in `src/tui/view/mod.rs`.

## Verification
- `mise run ci` passes.
- Manual smoke: opening an issue from the board shows metadata and body;
  `j`/`k` scrolls the body.
