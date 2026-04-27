---
# ish-778a
title: 'TUI: implement effect executor over core::store'
status: completed
type: task
priority: high
tags:
- tui
created_at: 2026-04-25T03:20:55.576306Z
updated_at: 2026-04-25T03:53:53.792437Z
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


## Implementation notes
- Replaced the TUI effect stub in `src/tui/effect.rs` with a store-backed executor: `execute(effect, &mut Store) -> Vec<Msg>` now handles load, create, save, editor-request, and quit flows without touching `crossterm`.
- `Effect::LoadIssues` reloads the store cache and emits `Msg::IssuesLoaded(...)` with cloned `Ish` values so downstream update/runtime code can stay pure.
- `Effect::SaveIssue` converts TUI status enums into `core::store::UpdateIsh`, preserves optimistic concurrency via `if_match`, maps ETag mismatches to `Msg::SaveFailed(Conflict { id })`, and emits a follow-up reload on success.
- `Effect::CreateIssue` converts the TUI draft into `core::store::CreateIsh`, reloads after creation, and emits `Msg::EditorRequested(...)` when `open_in_editor` is requested so the future runtime can own terminal/editor suspension.
- Added focused executor tests covering load, save+reload, stale-etag conflict reporting, create+editor follow-up, and the runtime marker messages for open-editor/quit.

## Validation
- `mise exec -- cargo test tui::effect -- --nocapture`
- `mise exec -- ish check`
- `mise run ci`

## Follow-up notes
- The executor currently takes `&mut Store` rather than `&Store` because `core::store` mutates its in-memory cache during `load`, `create`, and `update`; future runtime work should pass its owned store handle mutably through effect execution.
- Successful create/save execution currently emits `SaveCompleted` before the full reload message, and create-with-editor appends `EditorRequested` after that reload; keep that FIFO ordering in mind when wiring the runtime event queue so follow-up screens/status messages behave predictably.
