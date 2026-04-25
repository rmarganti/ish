---
# ish-jeza
title: 'TUI: create form view'
status: todo
type: task
priority: high
tags:
- tui
created_at: 2026-04-25T03:20:55.699020Z
updated_at: 2026-04-25T03:21:17.780Z
parent: ish-q6t1
blocked_by:
- ish-8dtp
- ish-5017
---

## Goal
Render the create-form screen with title, type, priority, and tags fields.

## Scope
### `src/tui/view/create_form.rs`
- `pub fn draw(frame: &mut Frame, area: Rect, model: &Model, state: &CreateFormState)`.
- Centered panel with four labeled rows + a submit row.
- Type and Priority rendered as `< value >` cycle widgets; show colors
  from `theme`.
- Tags shown as comma-separated text input.
- Highlight the focused field.
- Footer hints: `Tab/Shift-Tab field  Ctrl-s save  Ctrl-e save+edit  Esc cancel`.
- Confirm-on-cancel: when the update layer requests it, render a small
  modal asking "Discard new issue? y/n" (state for this lives in
  `CreateFormState` as `pending_cancel: bool`).

## Files
- `src/tui/view/create_form.rs`
- Add `pending_cancel: bool` to `CreateFormState` if not already present
  (coordinate with the types ish).

## Verification
- `mise run ci` passes.
- Manual smoke: `c` from the board opens the form; tab cycles fields;
  `Esc` with non-empty input shows the confirm; `Ctrl-s` creates an ish
  and the board reloads to show it.
