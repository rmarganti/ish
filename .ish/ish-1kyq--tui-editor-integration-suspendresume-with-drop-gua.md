---
# ish-1kyq
title: 'TUI: editor integration (suspend/resume with Drop guard)'
status: todo
type: task
priority: high
tags:
- tui
created_at: 2026-04-25T03:20:55.740080Z
updated_at: 2026-04-25T03:21:17.791894Z
parent: ish-q6t1
blocked_by:
- ish-twvz
---

## Goal
Implement the `$EDITOR` integration: resolve the editor binary, suspend
the TUI cleanly, run the editor, and restore the terminal — surviving
panics and non-zero exits.

## Scope
### `src/tui/runtime/editor.rs` (or `src/tui/editor.rs`)
- `pub fn open_editor(path: &Path) -> AppResult<()>`.
- Resolution order: `$VISUAL` → `$EDITOR` → `vi`.
- Suspend pattern:
  1. Leave alternate screen, disable raw mode, show cursor, flush.
  2. Spawn editor as a foreground child via `std::process::Command::status`.
  3. On return (any exit status), re-enter alternate screen, enable raw
     mode, hide cursor, redraw.
- Wrap step 1 → step 3 in a Drop guard so a panic inside the closure
  still restores TUI mode.
- Non-zero editor exit and spawn failure return `Err`; the runtime turns
  these into a status-line error rather than crashing.
- After return, runtime always emits `Effect::LoadIssues`.

## Files
- New module under `src/tui/runtime/` or as a sibling.

## Verification
- `mise run ci` passes.
- Manual smoke: from detail view, press `e`, edit the title in the
  editor, save and exit — the TUI returns and shows the new title.
- Manual smoke: set `EDITOR=false` and press `e` — TUI shows a red
  status-line error and stays usable.
