---
# ish-07aa
title: 'TUI: wire priority picker state and save flow'
status: completed
type: task
priority: high
tags:
- tui
created_at: 2026-04-27T20:51:27.111321459Z
updated_at: 2026-04-27T20:58:21.418872Z
parent: ish-4smz
---

## Context
This task covers the non-visual priority-picker flow so the TUI can open a priority-changing interaction from issue detail and persist the result through the same store-backed optimistic-concurrency path used by status changes.

The existing status flow is the reference implementation:
- `src/tui/keymap.rs`: detail-screen `s` binding and picker navigation bindings
- `src/tui/update.rs`: `Msg::OpenStatusPicker`, modal push/pop, `Msg::SubmitStatusChange`
- `src/tui/effect.rs`: `IssuePatch` -> `UpdateIsh` save path using the current ETag
- `src/tui/model.rs` / `src/tui/msg.rs`: screen, picker state, and message definitions

Priority differs slightly from status because the store models it as optional in some places, but the TUI already normalizes missing priority to `normal`. Keep this v1 flow simple: pick among the configured TUI `Priority` enum values and save a concrete priority rather than introducing an unset/none modal option.

## Dependencies
- Parent feature: `ish-4smz`.

## Work
- Extend the TUI state/message/effect types for priority picking:
  - add a dedicated priority-picker screen/state in `src/tui/model.rs`
  - add `Msg::OpenPriorityPicker` and `Msg::SubmitPriorityChange` in `src/tui/msg.rs`
  - extend `IssuePatch` in `src/tui/effect.rs` so saves can carry `priority`
- Wire issue-detail key handling and update flow:
  - map `p` in `src/tui/keymap.rs`
  - in `src/tui/update.rs`, push a priority picker seeded from the current issue priority
  - on submit, emit `Effect::SaveIssue` with the current ETag and pop the modal optimistically
- Update the effect executor so `Effect::SaveIssue` maps the selected TUI priority into `UpdateIsh.priority`.
- Add focused regression coverage for the new keymap/update/effect behavior.

## Verification
- `mise exec -- cargo test tui::keymap -- --nocapture`
- `mise exec -- cargo test tui::update -- --nocapture`
- `mise exec -- cargo test tui::effect -- --nocapture`
- `mise exec -- ish check`
- `mise run ci`


## Implementation notes
- Added a dedicated `PriorityPickerState` / `Screen::PriorityPicker(...)` path so priority changes no longer have to overload the existing status-picker state.
- Wired `p` from issue detail into the pure update flow with `Msg::OpenPriorityPicker` / `Msg::SubmitPriorityChange`; the picker seeds from the issue's current priority and treats missing stored priority as `normal`.
- Extended `tui::effect::IssuePatch` so `Effect::SaveIssue` can persist either status or priority updates through the existing optimistic-concurrency save path.
- Added focused regression coverage in `src/tui/keymap.rs`, `src/tui/update.rs`, and `src/tui/effect.rs` for the new binding, picker state transitions, and store-backed priority save behavior.

## Validation
- `mise exec -- cargo test tui::keymap -- --nocapture`
- `mise exec -- cargo test tui::update -- --nocapture`
- `mise exec -- cargo test tui::effect -- --nocapture`
- `mise exec -- ish check`
- `mise run ci`

## Follow-up notes
- `src/tui/view.rs` currently routes `Screen::PriorityPicker(...)` through the existing placeholder renderer so this task could land the non-visual state/save flow independently. `ish-2oss` should replace that placeholder with the real modal overlay and shared picker rendering/generalized layering work.
