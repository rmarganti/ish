---
# ish-wka9
title: 'TUI: footer, status line, and help overlay'
status: completed
type: task
priority: high
tags:
- tui
created_at: 2026-04-25T03:20:55.719797Z
updated_at: 2026-04-25T04:31:39.145636Z
parent: ish-q6t1
blocked_by:
- ish-8dtp
- ish-5017
---

## Goal
Implement the persistent footer (keybind hints), the status line, and
the `?` help overlay.

## Scope
### `src/tui/view/footer.rs`
- One-line footer rendered at the bottom of every screen.
- Content varies by top-of-stack screen (mirror keymap tables in the
  keymap ish).

### `src/tui/view/status_line.rs`
- Single-line widget rendered above the footer.
- Reads `model.status_line` and applies severity colors from `theme`.
- Hidden when `None`.

### `src/tui/view/help.rs`
- Full-screen overlay listing every binding grouped by screen.
- Pressing any key pops it (handled by keymap/update).

## Files
- `src/tui/view/footer.rs`, `status_line.rs`, `help.rs`
- Update `src/tui/view/mod.rs` to lay out: main area (top), status_line
  (height 1), footer (height 1).

## Verification
- `mise run ci` passes.
- Manual: trigger `r` and observe the transient "Refreshed" message that
  auto-dismisses; trigger a save conflict (e.g. by editing an ish from
  another shell mid-modal) and observe the sticky red error; press `?`
  to open the overlay and any key to close it.

## Implementation notes
- Added shared view modules under `src/tui/view/`:
  - `footer.rs` renders a single-line persistent footer with screen-specific
    key hints.
  - `status_line.rs` renders the severity-colored transient/sticky status
    message row from `model.status_line`.
  - `help.rs` renders the help overlay with grouped bindings for global,
    board, detail, status-picker, and create-form contexts.
- Updated `src/tui/view.rs` to reserve bottom layout rows for the shared
  status line + footer and dispatch `Screen::Help(...)` through the new help
  overlay renderer.
- Removed duplicated footer chrome from `src/tui/view/issue_detail.rs` and
  `src/tui/view/create_form.rs` so screen-specific views only render their
  main content.
- Added focused unit coverage for footer/help/status-line rendering and kept
  the existing create-form/detail render smoke tests passing after the shared
  layout change.

## Validation
- `mise exec -- cargo test`
- `mise run ci`
- `mise exec -- ish check`

## Follow-up notes
- `Screen::StatusPicker(...)` still uses the placeholder main-area renderer
  until `ish-icc4` lands, but it already benefits from the shared footer and
  status-line layout introduced here.
- Future overlays/screens should prefer updating `src/tui/view/footer.rs`
  for key-hint chrome instead of embedding footer rows inside individual view
  modules.
