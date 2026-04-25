---
# ish-q6t1
title: TUI Kanban board (ish tui)
status: todo
type: epic
priority: high
tags:
- tui
created_at: 2026-04-25T03:20:34.523964Z
updated_at: 2026-04-25T03:20:34.523964Z
---

# Epic: Kanban TUI for ish

Add an interactive `ish tui` subcommand that launches a ratatui+crossterm
terminal UI. Default screen is a four-column kanban board (`draft`, `todo`,
`in-progress`, `completed`). Vim-style navigation; opens issue detail,
status-picker modal, and create form; uses `$EDITOR` for substantive edits;
shares the data layer (`core::store`) with the existing CLI and honors the
same ETag-based optimistic concurrency.

## Source PRD
`.local/prds/1777086527-tui-kanban.md`

## Architecture (load-bearing)
Elm-style: `Model` + `Msg` + pure `update(Model, Msg) -> (Model, Vec<Effect>)`.
The runtime is the only thing that touches the terminal; the effect executor
is callable without crossterm (so integration tests can drive it).

Top-level layout:
- `src/tui/{model,msg,keymap,update,effect,runtime,view,theme}.rs`
- `src/commands/tui.rs` is a thin entry that calls `tui::run`.

## Children (see roadmap)
Foundation -> screens -> tests -> polish. Each child ish carries its own
context, file paths, and verification steps. Run `mise run ci` after every
change.

## Verification
- `ish tui` launches the board; `q`/`Ctrl-c` quits cleanly without leaving
  the terminal in raw mode.
- Saves through the TUI are visible via `ish list` immediately and vice
  versa, and ETag conflicts surface as user-visible errors.
- `mise run ci` is green.
