---
# ish--u501
title: 'TUI: create-form discard modal uses explicit y/n confirmation'
status: completed
type: task
priority: normal
tags:
- tui
parent: ish-q6t1
created_at: 2026-04-25T03:23:55.511548Z
updated_at: 2026-04-25T09:00:00Z
---

## Goal
Close the remaining PRD mismatch in the create-form cancel flow so the
visible discard-confirmation modal is backed by real `y` / `n` behavior
instead of the previous second-`Esc` shortcut.

## Scope
- Update create-form key handling so the discard modal accepts `y` to
  discard and `n`/`Esc` to keep editing.
- Update pure create-form update logic to clear or preserve modal state
  correctly and avoid leaking the discard prompt status line after the
  modal is resolved.
- Refresh shared TUI chrome/help copy so the footer/help overlay match
  the live behavior.
- Add focused regression tests for keymap, update, footer, and the modal
  renderer.

## Verification
- `mise exec -- cargo test tui::keymap -- --nocapture`
- `mise exec -- cargo test tui::update -- --nocapture`
- `mise exec -- cargo test tui::view::create_form -- --nocapture`
- `mise exec -- cargo test tui::view::footer -- --nocapture`
- `mise exec -- cargo test tui::view::help -- --nocapture`
- `mise run ci`

## Implementation notes
- Added explicit `Msg::ConfirmDiscardCreateForm` / `Msg::CancelDiscardCreateForm` messages so the modal flow is represented in the Elm-style update layer instead of overloading `PopScreen`.
- `src/tui/keymap.rs` now treats `CreateFormState::pending_cancel` as a true modal state: only `y`, `n`, `Esc`, global help, and `Ctrl-c` are handled while the prompt is open.
- `src/tui/update.rs` now sets the status line to `Discard new issue? y/n`, pops the form only on explicit confirmation, and clears the prompt/status line when the user keeps editing.
- Updated the create-form modal, footer, and help overlay copy so all visible affordances consistently advertise the real `y`/`n` flow.

## Validation
- `mise exec -- cargo test tui::keymap -- --nocapture`
- `mise exec -- cargo test tui::update -- --nocapture`
- `mise exec -- cargo test tui::view::create_form -- --nocapture`
- `mise exec -- cargo test tui::view::footer -- --nocapture`
- `mise exec -- cargo test tui::view::help -- --nocapture`
- `mise run ci`

## Follow-up notes
- The discard prompt now behaves like a proper modal. If later UX work adds literal button-like widgets or mouse support, preserve the current message-level contract so the modal remains testable through pure `keymap` + `update` coverage.
