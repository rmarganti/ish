---
# ish-a6jl
title: 'TUI: core::store load_one(id) helper'
status: todo
type: task
priority: high
tags:
- tui
created_at: 2026-04-25T03:20:55.863678Z
updated_at: 2026-04-25T03:20:55.863678Z
parent: ish-q6t1
---

## Goal
Add `core::store::load_one(id: &str)` so the post-editor reload path
can re-read a single issue and surface per-issue parse errors without a
full workspace re-scan.

## Scope
- `src/core/store.rs`: add `pub fn load_one(&self, id: &str) -> StoreResult<Ish>`.
- Implementation: locate the file by id prefix in `.ish/`, parse it,
  return the result. Return a typed not-found / parse error so callers
  can show a focused message.
- Wire the TUI runtime to use `load_one` after `OpenEditorForIssue`
  returns; on parse error, drop the issue from cache and surface a
  persistent warning in the status line.

## Files
- `src/core/store.rs`
- `src/tui/effect.rs` and/or `src/tui/runtime.rs` to use it.

## Notes
- This is **optional for v1** per the PRD's "Further Notes" — if it
  becomes complicated, the post-editor path can fall back to a full
  reload and this ish can stay open for a follow-up release.

## Verification
- `mise run ci` passes.
- Unit test in `core/store`: `load_one` returns Ok for a valid id, a
  not-found error for an unknown id, and a parse error for a corrupted
  file (write garbage to a tempfile and re-read).
