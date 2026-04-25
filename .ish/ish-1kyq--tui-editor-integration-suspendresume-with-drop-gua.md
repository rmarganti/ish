---
# ish-1kyq
title: 'TUI: editor integration (suspend/resume with Drop guard)'
status: completed
type: task
priority: high
tags:
- tui
created_at: 2026-04-25T03:20:55.740080Z
updated_at: 2026-04-25T04:29:22.381130Z
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

## Implementation notes
- Added `src/tui/editor.rs` as the new editor integration boundary for the TUI runtime. It resolves the editor from `$VISUAL` → `$EDITOR` → `vi`, parses shell-style editor commands with `shell-words`, and appends the issue file path as the final argument.
- The editor helper uses a dedicated `SuspendedTerminal` Drop guard to leave the alternate screen, show the cursor, disable raw mode, flush stdout, and then restore TUI mode even if launching the editor panics or returns early with an error.
- `src/tui/runtime.rs` now handles `Msg::EditorRequested(...)` by locating the issue file through the loaded store cache, opening the editor against the on-disk markdown file, converting the result into `Msg::EditorReturned(...)`, and always queueing a full `Effect::LoadIssues` reload afterward.
- Added `Store::root()` in `src/core/store.rs` so runtime/editor code can resolve issue file paths from the canonical store root instead of assuming the current working directory matches the project root.

## Validation
- `mise exec -- cargo test tui::editor -- --nocapture`
- `mise exec -- cargo test`
- `mise run ci`
- `mise exec -- ish check`

## Follow-up notes
- Editor command parsing now supports quoted executable names and flags such as `code --wait`; if future work needs shell-only forms like `EDITOR='env VAR=1 vim'`, extend `src/tui/editor.rs` at the parser boundary instead of duplicating command resolution in runtime.
- Post-editor refresh still uses the PRD-approved full workspace reload path. If a later task wants per-issue parse-error recovery, the existing `Store::load_one(...)` helper is the right place to plug that in.
