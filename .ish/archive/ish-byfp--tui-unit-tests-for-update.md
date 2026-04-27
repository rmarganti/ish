---
# ish-byfp
title: 'TUI: unit tests for update'
status: completed
type: task
priority: high
tags:
- tui
created_at: 2026-04-25T03:20:55.802174Z
updated_at: 2026-04-25T04:46:44.468746Z
parent: ish-q6t1
blocked_by:
- ish-yfuo
- ish-fww3
---

## Goal
Comprehensive unit tests for `tui::update`, covering every PRD-mandated
behavior of the pure update function.

## Scope
- Place tests in `src/tui/update.rs` `#[cfg(test)] mod tests` (or a
  sibling `update_tests.rs` if preferred).
- Use the `dispatch` helper from the test_support ish.

### Coverage
- **Board navigation**: no-wrap at edges (left of col 0, right of col 3,
  top/bottom of column), per-column cursor memory across horizontal
  moves, empty columns navigable but no card selected, `g`/`G`,
  `Ctrl-d`/`Ctrl-u` half-page semantics.
- **Bucketing**: `IssuesLoaded` repopulates buckets and clamps cursors
  to new sizes; archived/scrapped excluded.
- **Screen transitions**: `enter` opens detail, `s` pushes status picker
  on top of detail, `q`/`Esc` pops correctly, `c` opens create form.
- **Status save flow**: SubmitStatusChange emits `Effect::SaveIssue` with
  the right etag and pops modal; `SaveCompleted` triggers a follow-up
  `Effect::LoadIssues`; `SaveFailed(Conflict)` sets a sticky error and
  does not pop the screen stack further.
- **Status-line lifecycle**: info/success cleared by `Tick` after ~3s;
  error stays through `Tick` and only clears on `DismissStatusLine`;
  errors cannot be replaced by info/success during the 1s sticky window.
- **CreateForm**: tab cycles fields; type/priority cycling wraps;
  text input appends; `Esc` with non-empty fields sets pending_cancel
  rather than popping.
- **Quit**: `Ctrl-c` sets `model.quit = true` regardless of screen.

## Files
- `src/tui/update.rs` (#[cfg(test)] block) or `src/tui/update_tests.rs`.

## Verification
- `mise run ci` passes; tests run as part of `mise run test`.
- All assertions are on returned `(Model, Vec<Effect>)`, not on
  internal helpers.


## Implementation notes
- Expanded `src/tui/update.rs` test coverage from a single smoke test to a broad pure-update suite that exercises board navigation, issue reload clamping, screen-stack transitions, status-save flows, status-line expiration/stickiness, create-form behavior, and quit handling.
- Added local test helpers for seeded board models plus screen/board-state assertions so the update tests stay focused on `(Model, Vec<Effect>)` outcomes instead of internal helper functions.
- Captured the current save contract explicitly in tests: `SaveCompleted(...)` updates the success status line while the follow-up reload continues to be driven by the effect executor's emitted `Msg::IssuesLoaded(...)`.

## Validation
- `mise exec -- cargo test tui::update -- --nocapture`
- `mise run ci`
