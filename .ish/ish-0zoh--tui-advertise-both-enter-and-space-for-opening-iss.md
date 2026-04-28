---
# ish-0zoh
title: 'TUI: advertise both Enter and Space for opening issue details'
status: completed
type: bug
priority: normal
tags:
- tui
created_at: 2026-04-28T14:47:36.717952003Z
updated_at: 2026-04-28T14:55:09.498162Z
---

## Context
In the issue list view, both `Enter` and `Space` open the selected issue detail, but the visible keybind hints only advertise `enter`.

## Goal
Update the TUI help/footer copy so it reports both supported keybinds for opening issue details.

## Verification
- The issue list/footer/help text advertises both `enter` and `space` for opening issue details.
- `mise run ci` passes.



## Implementation notes
- Updated `src/tui/view/footer.rs` so the board footer now advertises `enter/space open`, matching the existing keymap behavior and the help overlay copy.
- Tightened the footer regression test to assert the full board footer string so this hint does not silently regress back to enter-only copy.

## Validation
- `mise exec -- cargo test tui::view::footer -- --nocapture`
- `mise exec -- cargo test tui::view::help -- --nocapture`
- `mise exec -- ish check`
- `mise run ci`

## Follow-up notes
- The help overlay already documented `Enter / Space`; this fix brings the board footer into parity with the existing keymap + help behavior.
