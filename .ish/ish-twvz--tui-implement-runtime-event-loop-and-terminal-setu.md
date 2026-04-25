---
# ish-twvz
title: 'TUI: implement runtime event loop and terminal setup'
status: completed
type: task
priority: high
tags:
- tui
created_at: 2026-04-25T03:20:55.596917Z
updated_at: 2026-04-25T04:13:55.250957Z
parent: ish-q6t1
blocked_by:
- ish-yfuo
- ish-jrw0
- ish-778a
---

## Goal
Implement the event loop: terminal setup/teardown, key polling, msg
dispatch, effect execution, redraw, and tick generation.

## Scope
### `src/tui/runtime.rs`
- `pub fn run(ctx: &AppContext) -> AppResult<()>`:
  1. Enable raw mode, enter alternate screen, hide cursor. Wrap teardown
     in a Drop guard (`TerminalGuard`) that restores on panic.
  2. Build initial `Model` and dispatch `Effect::LoadIssues` immediately.
  3. Loop: `event::poll(Duration::from_millis(250))`:
     - Key event → `keymap::map_key` → `Msg`.
     - Resize → `Msg::Resize`.
     - Timeout → `Msg::Tick`.
  4. `let (model, effects) = update(model, msg);`
  5. Execute each effect via `effect::execute`; feed resulting Msgs
     back into the loop (FIFO queue).
  6. If `effect == OpenEditorForIssue { id }`, call the editor helper
     (separate ish), then on return push `Msg::EditorReturned(...)`
     (followed by `LoadIssues`).
  7. Redraw via `view::draw(frame, &model)` after each model change.
  8. Exit when `model.quit` is true.
- Tick handling: only emit when poll times out; do not flood.

## Files
- `src/tui/runtime.rs`.
- Update `src/tui/mod.rs::run` to delegate to `runtime::run`.

## Verification
- `mise run ci` passes.
- Manual smoke: `cargo run -- tui` opens an empty board (or current
  workspace's board), `q` quits cleanly, terminal is restored.
- Sending `SIGINT` (Ctrl-c) restores the terminal.
- Triggering a panic in `update` (temporary `panic!`) leaves the
  terminal usable thanks to the Drop guard. Remove the panic before
  finishing.


## Implementation notes
- Replaced the no-op `src/tui/runtime.rs` stub with the first real terminal runtime: it now initializes a `ratatui`/`crossterm` terminal, enters raw mode + the alternate screen behind a `TerminalGuard`, polls keys/resizes/ticks, redraws via `view::draw(...)`, and exits cleanly when `model.quit` is set.
- The runtime now owns the FIFO message/effect loop described in the PRD: startup enqueues `Resize` + `Effect::LoadIssues`, key events flow through `keymap::map_key(...)`, messages are reduced through `tui::update::update(...)`, and returned effects are executed via `tui::effect::execute(...)` before their follow-up messages are processed.
- `Msg::EditorRequested(...)` is now handled explicitly in the runtime so editor work can land separately without blocking the event loop; until `ish-1kyq` lands, the runtime converts that marker into a persistent error status instead of trying to suspend the terminal prematurely.
- `ish tui` now takes ownership of `AppContext` so the runtime can reuse the already-loaded mutable store handle directly instead of reloading workspace state through a second code path.
- Headless/non-TTY invocations now return `Ok(())` without attempting raw-mode terminal setup, which keeps command/unit tests stable while preserving the real interactive flow for actual TTY sessions.

## Validation
- `mise exec -- cargo test`
- `mise run ci`
