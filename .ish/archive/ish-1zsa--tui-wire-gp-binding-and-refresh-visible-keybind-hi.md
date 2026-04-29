---
# ish-1zsa
title: 'TUI: wire gp binding and refresh visible keybind hints'
status: completed
type: task
priority: normal
tags:
- tui
created_at: 2026-04-28T17:44:25.963930Z
updated_at: 2026-04-28T18:11:15.823733Z
parent: ish-8ie2
blocked_by:
- ish-z15w
- ish-2boi
---

## Context
After the input pipeline can handle prefixes and the update layer understands a go-to-parent action, we still need to expose the final user-facing binding and make the visible TUI affordances truthful.

The agreed UX is:
- `gg` -> jump to top
- `G` -> jump to bottom
- `gp` -> go to parent

Today the help/footer/tests still describe the older single-key `g` jump-top behavior, so this final integration pass needs to bring implementation, docs, and tests back into sync.

## Dependencies
- Blocked by the stateful multi-key input pipeline task.
- Blocked by the go-to-parent action task.
- Parent feature: the new multi-key TUI navigation feature.

## Work
- Wire `gp` to the go-to-parent action on the relevant TUI screens.
- Make sure board/detail help text and footer hints advertise the final bindings accurately.
- Update regression tests that currently pin `g` as jump-top so they instead pin the new `gg` / `gp` behavior.
- Verify there are no stale docs/help strings that still imply single-key `g` is jump-top.

## Verification
- Keymap tests assert `gp` resolves to the new parent-navigation action and `gg` resolves to jump-top.
- Footer/help view tests assert the visible binding hints mention the final behavior.
- `mise exec -- cargo test tui::keymap -- --nocapture`
- `mise exec -- cargo test tui::view::help -- --nocapture`
- `mise exec -- cargo test tui::view::footer -- --nocapture`
- `mise run ci`


## Implementation notes
- Added `gp` as a second `g`-prefix sequence in `src/tui/keymap.rs` for both board and detail screens, dispatching the existing semantic `Msg::GoToParent` action without changing the shared prefix resolver.
- Updated `src/tui/view/help.rs` so board and detail help now advertise `gg / G` for top/bottom navigation and `gp` for parent navigation.
- Updated `src/tui/view/footer.rs` and its regression tests so the visible footer hints stay aligned with the live multi-key navigation behavior.

## Validation
- `mise exec -- cargo test tui::keymap -- --nocapture`
- `mise exec -- cargo test tui::view::help -- --nocapture`
- `mise exec -- cargo test tui::view::footer -- --nocapture`
- `mise run ci`
- `mise exec -- ish check`

## Follow-up notes
- The `g` namespace now has two stable sequences (`gg`, `gp`). If more `g*` navigation is added later, extend the binding tables/help copy together so the prefix remains discoverable and documented.
