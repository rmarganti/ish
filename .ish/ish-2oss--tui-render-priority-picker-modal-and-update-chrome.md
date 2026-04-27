---
# ish-2oss
title: 'TUI: render priority picker modal and update chrome'
status: completed
type: task
priority: high
tags:
- tui
created_at: 2026-04-27T20:51:38.077861190Z
updated_at: 2026-04-27T21:02:42.979013Z
parent: ish-4smz
blocked_by:
- ish-07aa
---

## Context
This task covers the visible TUI UX for changing priority once the priority-picker screen/state exists. The current status-picker renderer in `src/tui/view/status_picker.rs` and the current overlay special case in `src/tui/view.rs` are the main references.

The implementation recommendation from investigation was to avoid a second one-off detail+modal special case. Instead, introduce a small generalization so a priority picker can reuse the same overlay pattern cleanly and future modal pickers do not keep expanding `src/tui/view.rs` with bespoke matches.

## Dependencies
- Parent feature: `ish-4smz`.
- Blocked by `ish-07aa` so the priority-picker screen/messages/save path already exist.

## Work
- Add the priority-picker modal renderer, either by:
  - creating `src/tui/view/priority_picker.rs`, or
  - generalizing `src/tui/view/status_picker.rs` into a shared picker renderer with priority/status-specific titles and row styling.
- Update `src/tui/view.rs` so the picker overlays are rendered as true modals on top of issue detail without another hard-coded one-off branch.
- Update shared TUI chrome to advertise the new behavior:
  - issue-detail footer in `src/tui/view/footer.rs` should show `p priority`
  - help copy in `src/tui/view/help.rs` should document the new detail binding and picker controls
- Add focused view/help/footer regression tests covering the new modal and overlay path.

## Verification
- `mise exec -- cargo test tui::view::status_picker -- --nocapture`
- `mise exec -- cargo test tui::view::footer -- --nocapture`
- `mise exec -- cargo test tui::view::help -- --nocapture`
- `mise exec -- ish check`
- `mise run ci`


## Implementation notes
- Added a shared modal helper in `src/tui/view/picker_modal.rs` plus a new `src/tui/view/priority_picker.rs` so status and priority pickers reuse the same centered modal chrome while keeping their row labeling and coloring specific to the selected field.
- Generalized `src/tui/view.rs` from a one-off detail+status match into stack-based rendering: full-screen screens draw in order, and picker screens now overlay on top of the already-rendered detail screen without another bespoke branch.
- Updated `src/tui/view/footer.rs` and `src/tui/view/help.rs` so issue detail advertises `p priority` and the help overlay documents both the new detail binding and the priority-picker controls.
- Added focused regression coverage for the new priority picker view plus the updated footer/help text, and kept the existing status-picker render smoke tests green through the shared modal refactor.

## Validation
- `mise exec -- cargo test tui::view::status_picker -- --nocapture`
- `mise exec -- cargo test tui::view::priority_picker -- --nocapture`
- `mise exec -- cargo test tui::view::footer -- --nocapture`
- `mise exec -- cargo test tui::view::help -- --nocapture`
- `mise exec -- ish check`
- `mise run ci`
