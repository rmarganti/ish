---
# ish-4smz
title: 'TUI: detail-view priority picker'
status: completed
type: feature
priority: high
tags:
- tui
created_at: 2026-04-27T20:51:12.929269442Z
updated_at: 2026-04-27T21:02:58.562963Z
---

## Context
In the TUI there is currently no easy way to change issue priority. Status already has a good interaction pattern from the issue detail screen: pressing `s` opens a modal picker, the user moves through options with `j`/`k` or arrows, `Enter` saves, and `q`/`Esc` cancels.

We want the priority-changing workflow to feel the same. Investigation in the current codebase shows the existing status flow spans `src/tui/keymap.rs`, `src/tui/update.rs`, `src/tui/effect.rs`, `src/tui/model.rs`, `src/tui/msg.rs`, `src/tui/view/status_picker.rs`, `src/tui/view.rs`, `src/tui/view/footer.rs`, and `src/tui/view/help.rs`.

The recommended approach is to add a detail-view `p` binding for priority, use a picker modal with the same interaction model as the status picker, and take this opportunity to remove the current detail+status-picker overlay special case in `src/tui/view.rs` so another modal picker fits naturally.

## Dependencies
- None.

## Work
- Add TUI support for changing priority from the issue detail screen.
- Keep the UX parallel with the existing status-picker flow.
- Preserve the current store-backed optimistic-concurrency save path.
- Prefer a small generalization of picker/modal rendering over copy-pasting more status-specific logic.
- Track the work in child tasks covering the state/save flow and the modal rendering/chrome updates.

## Verification
- [x] Child tasks are complete.
- [x] From issue detail, `p` opens a priority picker, navigation matches the status picker, `Enter` saves, and `q`/`Esc` cancels.
- [x] `mise run ci` passes.

## Implementation notes
- Child task `ish-07aa` landed the non-visual message/state/save path for priority changes, including `p` key handling in detail, a dedicated `PriorityPicker` screen state, and store-backed priority persistence through `Effect::SaveIssue`.
- Child task `ish-2oss` finished the visible UX with a real priority-picker modal, a shared picker modal renderer, generalized stacked-screen rendering in `src/tui/view.rs`, and updated footer/help chrome so the new interaction is discoverable.
- The resulting flow now mirrors status changes end to end: from issue detail, `p` opens a modal overlay, picker navigation uses the same controls as status, and submit/cancel behavior stays on the existing optimistic-concurrency save path.

## Validation
- `mise exec -- cargo test tui::keymap -- --nocapture`
- `mise exec -- cargo test tui::update -- --nocapture`
- `mise exec -- cargo test tui::effect -- --nocapture`
- `mise exec -- cargo test tui::view::status_picker -- --nocapture`
- `mise exec -- cargo test tui::view::priority_picker -- --nocapture`
- `mise exec -- cargo test tui::view::footer -- --nocapture`
- `mise exec -- cargo test tui::view::help -- --nocapture`
- `mise exec -- ish check`
- `mise run ci`
