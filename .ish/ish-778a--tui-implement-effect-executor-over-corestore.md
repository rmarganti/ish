---
# ish-778a
title: 'TUI: implement effect executor over core::store'
status: todo
type: task
priority: high
tags:
- tui
created_at: 2026-04-25T03:20:55.576306Z
updated_at: 2026-04-25T03:21:17.728568Z
parent: ish-q6t1
blocked_by:
- ish-8dtp
---

## Goal
Implement the `Effect` executor that wraps `core::store` and converts
results into `Msg`s. Must be callable independently of crossterm so
integration tests can drive it.

## Scope
### `src/tui/effect.rs`
- `pub fn execute(effect: Effect, store: &Store) -> Vec<Msg>`.
- `Effect::LoadIssues` → reads the workspace; emits
  `Msg::IssuesLoaded(Ok(...))` or `Err(...)`.
- `Effect::SaveIssue { patch, etag }` → updates via `core::store` with
  ETag; on conflict emits `Msg::SaveFailed(Conflict { id })`; on success
  emits `Msg::SaveCompleted` then `LoadIssues`.
- `Effect::CreateIssue { draft, open_in_editor }` → creates an ish; if
  `open_in_editor`, follow with `Effect::OpenEditorForIssue { id }`
  (the executor returns the chained effect's results too — or returns
  the additional effect as a `Msg::Followup(Effect)` handled by runtime;
  pick whichever is simplest given the runtime design).
- `Effect::OpenEditorForIssue { id }` lives in the `runtime` ish, not
  here — the executor returns a marker `Msg::EditorRequested(id)` that
  the runtime handles. (Document this in the module rustdoc.)
- `Effect::Quit` → sets a sentinel `Msg` the runtime acts on.

## Files
- `src/tui/effect.rs`.

## Notes
- If `core::store` lacks a single-issue load, fall back to a full
  `LoadIssues` after every mutation (acceptable for v1 per the PRD).

## Verification
- `mise run ci` passes.
- Integration smoke tests live in their own ish (depends on this).
