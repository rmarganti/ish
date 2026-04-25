---
# ish-jrw0
title: 'TUI: implement keymap (per-screen key->Msg)'
status: todo
type: task
priority: high
tags:
- tui
created_at: 2026-04-25T03:20:55.555644Z
updated_at: 2026-04-25T03:21:17.724802Z
parent: ish-q6t1
blocked_by:
- ish-8dtp
---

## Goal
Implement the pure context-aware key mapping from `crossterm::KeyEvent` to
`Option<Msg>` based on the current top-of-stack screen.

## Scope
### `src/tui/keymap.rs`
- `pub fn map_key(screen: &Screen, key: KeyEvent) -> Option<Msg>`.
- Global bindings checked first: `Ctrl-c` → `Quit`, `?` → `OpenHelp`
  (where applicable), `Esc` → `DismissStatusLine` if status line is an
  error and no modal handles it (handled by update).
- Per-screen tables:
  - **Board:** `h/j/k/l` + arrows → MoveX; `g/G`, `Ctrl-d/Ctrl-u`,
    `enter`/`space` → OpenDetail; `c` → OpenCreateForm; `r` → RequestRefresh;
    `q` → Quit.
  - **Detail:** `j/k/Ctrl-d/Ctrl-u/g/G` → body scroll; `e` →
    EditCurrentIssue; `s` → OpenStatusPicker; `q`/`Esc` → PopScreen.
  - **StatusPicker:** `j/k`, arrows, `Ctrl-n/Ctrl-p` → MoveDown/Up;
    `enter` → SubmitStatusChange; `q`/`Esc` → PopScreen.
  - **CreateForm:** `Tab`/`Shift-Tab`/`Ctrl-n`/`Ctrl-p` → focus cycling;
    `Ctrl-s`/`enter` (on submit) → SubmitCreateForm; `Ctrl-e` →
    SubmitCreateAndEdit; printable chars → CreateFormInput;
    `Esc` → PopScreen (update layer handles confirm prompt).
  - **Help:** any key → PopScreen.
- Keymap is pure and infallible — it never touches `Model`.

## Files
- `src/tui/keymap.rs`.

## Verification
- `mise run ci` passes.
- Unit tests covered separately (per-screen key→msg tables) in the keymap
  unit-test ish that depends on this one.
