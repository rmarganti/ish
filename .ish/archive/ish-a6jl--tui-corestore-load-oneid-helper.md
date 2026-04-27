---
# ish-a6jl
title: 'TUI: core::store load_one(id) helper'
status: completed
type: task
priority: high
tags:
- tui
created_at: 2026-04-25T03:20:55.863678Z
updated_at: 2026-04-25T03:49:15.376329Z
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

## Implementation notes
- `src/core/store.rs` now exposes `Store::load_one(&self, id: &str) -> Result<Ish, StoreError>`.
- `load_one(...)` normalizes short ids with the configured prefix, recursively scans the store root for a matching markdown file, skips hidden directories, and parses the matching file through the existing `load_ish(...)` path.
- The helper resolves archived issues too, which keeps the targeted reload path flexible for future runtime/editor work.
- Typed `StoreError::NotFound(...)` and `StoreError::Yaml { .. }` failures are preserved so callers can surface focused reload/parse messages.

## Validation
- `mise run ci`
- `mise exec -- ish check`

## Follow-up notes
- The runtime/editor-specific `load_one(...)` integration is still deferred because `src/tui/runtime.rs` is not implemented yet; once the editor flow lands, prefer the focused reload path first and fall back to full reload only when needed.
