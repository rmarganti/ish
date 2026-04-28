---
# ish-2boi
title: 'TUI: go-to-parent navigation action for board and detail'
status: todo
type: task
priority: high
tags:
- tui
created_at: 2026-04-28T17:44:25.825014Z
updated_at: 2026-04-28T17:44:26.111217Z
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
