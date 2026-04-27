---
# ish-5017
title: 'TUI: theme module and shared style helpers'
status: completed
type: task
priority: high
tags:
- tui
created_at: 2026-04-25T03:20:55.617096Z
updated_at: 2026-04-25T03:47:02.333742Z
parent: ish-q6t1
blocked_by:
- ish-loy6
---

## Goal
Centralize the TUI color palette and reuse the existing CLI palette so
the TUI feels like the same product.

## Scope
### `src/tui/theme.rs`
- Map every status (`draft`, `todo`, `in-progress`, `completed`,
  `scrapped`), every type (`milestone`, `epic`, `bug`, `feature`,
  `task`), and every priority (`critical`, `high`, `normal`, `low`,
  `deferred`) to a `ratatui::style::Color`.
- Source colors from `src/output/` so the two views stay in sync.
  Add a small adapter if `output/` exposes `colored::Color` rather than
  `ratatui::style::Color`.
- Severity colors for the status line (`Info` = dim grey, `Success` =
  green, `Error` = bold red).
- Style helpers: `card_border(selected: bool, focused_column: bool)`,
  `column_header(active: bool)`, `footer_key()`, `footer_desc()`.

## Files
- `src/tui/theme.rs`
- Possibly minor refactor in `src/output/` to expose color constants.

## Verification
- `mise run ci` passes.
- Theme is consumed by view ishes. A tiny snapshot is acceptable: a unit
  test that asserts the four kanban-column colors are non-default and
  match the `ish list` palette.


## Implementation notes
- Replaced the TUI theme stub with a shared palette module in `src/tui/theme.rs` that now exposes config-backed status/type/priority colors plus `status_style`, `type_style`, `priority_style`, `severity_style`, `card_border`, `column_header`, `footer_key`, and `footer_desc` helpers.
- Added `output::color_name_to_ratatui(...)` in `src/output/mod.rs` and made the existing color-name mapping reusable so the TUI and CLI now resolve the same configured palette names through one bridge.
- Added theme tests that pin the four kanban-column colors to the CLI palette, assert type/priority mappings, and cover the shared widget/status-line styling modifiers.

## Validation
- `mise run ci`
- `mise exec -- ish check`

## Follow-up notes
- Future view work should import styling from `crate::tui::theme` instead of hard-coding `ratatui` colors so later palette tweaks stay centralized.
