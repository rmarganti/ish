---
# ish-z15w
title: 'TUI: stateful key-sequence input pipeline'
status: todo
type: task
priority: high
tags:
- tui
created_at: 2026-04-28T17:44:25.682965Z
updated_at: 2026-04-28T17:44:26.096Z
parent: ish-8ie2
blocking:
- ish-1zsa
---

## Context
Current TUI key handling is single-event and stateless:
- `src/tui/runtime.rs` reads a `KeyEvent` and immediately asks `keymap::map_key(...)` for a `Msg`
- `src/tui/keymap.rs` matches one `KeyEvent` at a time
- `src/tui/model.rs` has no input/prefix state

That architecture cannot express multi-key bindings like `gp` and also makes prefix behavior awkward to evolve.

The agreed design is to keep input state in the model and resolve keys inside the update loop rather than hiding mutable key-sequence state in `runtime`.

## Dependencies
- Parent feature: the new multi-key TUI navigation feature.

## Work
- Add a raw keypress message path, likely `Msg::KeyPressed(KeyEvent)`, so `runtime` stops converting key events directly into semantic messages.
- Add input state to `Model` for pending key-sequence prefixes.
- Refactor `src/tui/keymap.rs` into a stateful resolver that can distinguish:
  - exact matches
  - prefix matches that should keep waiting
  - invalid continuations that should clear/retry cleanly
- Prefer declarative per-screen binding tables over a growing ad hoc `match` tree.
- Convert existing `g` jump-top bindings on board/detail screens into the prefix-friendly shape so `gg` can become jump-top without timeout logic.
- Preserve existing global behavior like `Ctrl-c` quit and help handling.

## Verification
- Focused keymap/update tests cover:
  - single-key bindings still working
  - `g` becoming pending where appropriate
  - `gg` resolving to `JumpTop`
  - invalid continuations clearing pending state correctly
- `mise exec -- cargo test tui::keymap -- --nocapture`
- `mise exec -- cargo test tui::update -- --nocapture`
- `mise run ci`
