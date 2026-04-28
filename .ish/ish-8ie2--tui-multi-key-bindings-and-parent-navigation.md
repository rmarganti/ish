---
# ish-8ie2
title: 'TUI: multi-key bindings and parent navigation'
status: todo
type: feature
priority: high
tags:
- tui
created_at: 2026-04-28T17:44:25.495743Z
updated_at: 2026-04-28T17:44:25.495743Z
---

## Context
We want the TUI keybind system to support multi-key sequences cleanly and idiomatically. Today key handling is stateless: `runtime` reads a single `KeyEvent`, immediately calls `keymap::map_key(screen, key)`, and gets back one `Msg`. There is no pending-prefix state, no sequence buffer, and no way to represent bindings like `gp`.

The agreed direction is:
- move key handling into the Elm-style update loop via `Msg::KeyPressed(KeyEvent)`
- add explicit input state to `Model`
- replace the current one-key-only mapping with a maintainable, declarative resolver that can handle prefixes and full matches
- make `g` a prefix namespace rather than introducing timeout-based ambiguity

The target UX for this feature is:
- `gg` -> jump to top
- `G` -> jump to bottom (unchanged)
- `gp` -> go to parent

## Work
- Add a stateful key-sequence input pipeline for the TUI.
- Add an explicit `GoToParent` navigation action that works in board/detail screens.
- Update visible help/footer copy and regression coverage so the documented bindings match the implementation.

## Verification
- The TUI can resolve multi-key bindings without timeout logic.
- `gg` works anywhere `g` previously meant jump-to-top.
- `gp` navigates to the current issue's parent from both board and detail views.
- `mise run ci` passes.
