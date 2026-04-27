---
# ish-jeza
title: 'TUI: create form view'
status: completed
type: task
priority: high
tags:
- tui
created_at: 2026-04-25T03:20:55.699020Z
updated_at: 2026-04-25T04:23:34.841805Z
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



## Implementation notes
- Added `src/tui/view/create_form.rs` and registered `Screen::CreateForm(...)` in `src/tui/view.rs`, so create-form navigation now renders a dedicated centered panel instead of the generic placeholder screen.
- The create form now renders labeled rows for title, type, priority, tags, and save, with focused-row highlighting plus `< value >` cycle widgets that reuse `theme::type_style(...)` and `theme::priority_style(...)`.
- Added a create-form footer hint row covering field navigation, save, save+edit, and cancel shortcuts.
- When `CreateFormState::pending_cancel` is set, the view now overlays a confirmation modal with the requested discard prompt instead of relying solely on the transient status line.
- Added focused create-form view coverage for placeholder/cycle formatting and a `TestBackend` render smoke test.

## Validation
- `mise exec -- cargo test tui::view::create_form -- --nocapture`
- `mise exec -- ish check`
- `mise run ci`

## Follow-up notes
- The create-form confirmation modal is now rendered, but the current update/keymap flow still treats a second `Esc` as the discard confirmation path; if later UX work wants literal `y`/`n` handling, wire that into `src/tui/keymap.rs` and `src/tui/update.rs` rather than duplicating modal state in the view.
