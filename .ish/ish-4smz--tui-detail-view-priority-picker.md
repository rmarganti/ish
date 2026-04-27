---
# ish-4smz
title: 'TUI: detail-view priority picker'
status: todo
type: feature
priority: high
tags:
- tui
created_at: 2026-04-27T20:51:12.929269442Z
updated_at: 2026-04-27T20:51:12.929269442Z
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
- [ ] Child tasks are complete.
- [ ] From issue detail, `p` opens a priority picker, navigation matches the status picker, `Enter` saves, and `q`/`Esc` cancels.
- [ ] `mise run ci` passes.
