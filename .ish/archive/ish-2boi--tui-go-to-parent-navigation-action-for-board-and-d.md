---
# ish-2boi
title: 'TUI: go-to-parent navigation action for board and detail'
status: completed
type: task
priority: high
tags:
- tui
created_at: 2026-04-28T17:44:25.825014Z
updated_at: 2026-04-28T18:08:13.396958Z
parent: ish-8ie2
blocking:
- ish-1zsa
---

## Context
Once the TUI can represent multi-key bindings, we need an explicit semantic action for parent navigation rather than burying the behavior inside keymap code.

Relevant existing helpers/paths:
- `src/tui/update.rs` already knows how to open detail screens and locate selected board issues.
- `src/tui/model.rs` board bucketing/tree rows already carry parent-child structure per status column.
- `Ish.parent` is already populated in the store/model layer.

The new action should work in both major navigation contexts:
- detail view: replace the current detail screen with the parent issue's detail screen
- board view: move selection to the parent issue, including switching columns if the parent lives in a different status bucket

This task should introduce the behavior as a first-class message/action so it is testable independently of the eventual `gp` binding.

## Dependencies
- Parent feature: the new multi-key TUI navigation feature.

## Work
- Add a dedicated navigation message/action for going to a parent issue.
- In detail view, navigate to the parent detail screen when present.
- In board view, select the parent issue in the correct status column and make it visible.
- Add or extract helpers as needed (for example a `select_issue_on_board(...)` helper) so board selection logic stays maintainable.
- Handle missing-parent cases gracefully with a useful status line instead of panicking.

## Verification
- Focused update tests cover:
  - detail view with parent -> opens parent detail
  - detail view without parent -> shows a clear status message
  - board view with parent -> selects the parent row/column
  - board view without parent -> leaves selection stable and reports the problem clearly
- `mise exec -- cargo test tui::update -- --nocapture`
- `mise run ci`



## Implementation notes
- Added a dedicated `Msg::GoToParent` semantic navigation action in `src/tui/msg.rs` so parent navigation is testable independently of any final key binding.
- `src/tui/update.rs` now handles parent navigation in both board and detail contexts: detail replaces the current issue detail with the parent and resets scroll, while board selection jumps across columns to the parent row and keeps it visible.
- Extracted `select_issue_on_board(...)` so board-side parent selection stays localized instead of duplicating bucket/cursor logic inline.
- Missing-parent cases now surface friendly info status lines instead of panicking, including the distinction between a parent that is missing from the cache and one that exists but is not visible on the board.
- Added focused update regression tests covering detail success/failure and board success/failure flows.

## Validation
- `mise exec -- cargo test tui::update -- --nocapture`
- `mise exec -- ish check`
- `mise run ci`

## Follow-up notes
- The semantic `GoToParent` action is now in place for the final `gp` binding task (`ish-1zsa`); that follow-up should only need keymap/help/footer wiring on top of this update-layer behavior.
