---
# ish-rlqk
title: 'TUI: integration smoke tests for effect executor'
status: completed
type: task
priority: high
tags:
- tui
created_at: 2026-04-25T03:20:55.843791Z
updated_at: 2026-04-25T04:39:36.707817Z
parent: ish-q6t1
blocked_by:
- ish-778a
- ish-fww3
---

## Goal
End-to-end smoke tests that drive the effect executor (no crossterm)
against a `tempfile`-backed `.ish/` directory and assert resulting files.

## Scope
- New file: `tests/tui_effects.rs` (Cargo integration test) or under
  `src/tui/` if internal-test patterns are used elsewhere — match what
  the existing project does.
- Test 1: create an issue via `Effect::CreateIssue`, then load via
  `Effect::LoadIssues`, assert it appears with the right fields.
- Test 2: save a status change via `Effect::SaveIssue`; assert the file
  on disk now has the new status and an updated etag.
- Test 3: simulate an external write (touch the file with new content
  under the hood) then attempt a `SaveIssue` with the stale etag;
  assert `Msg::SaveFailed(Conflict { id })` is returned.

## Files
- `tests/tui_effects.rs` (or chosen location).
- May need to expose `effect::execute` and a small store-construction
  helper in `test_support`.

## Verification
- `mise run ci` passes.
- Tests run via `mise run test`.

## Implementation notes
- Added `src/tui/effect_integration.rs` and registered it from `src/tui/mod.rs` under `#[cfg(test)]` so the smoke coverage can exercise crate-internal TUI/store types without first splitting the binary crate into a separate library target.
- Added three store-backed smoke tests that drive `tui::effect::execute(...)` against a temp `.ish/` workspace and assert both emitted `Msg`s and the markdown files written to disk.
- The conflict smoke test simulates an external write by overwriting the issue file on disk and reloading the store before attempting a save with the stale TUI etag, which matches the executor/runtime contract once external changes have been observed.

## Validation
- `mise exec -- cargo test tui::effect_integration -- --nocapture`
- `mise run ci`
- `mise exec -- ish check`

## Follow-up notes
- These smoke tests currently live alongside the TUI modules instead of under `tests/` because the project is still a binary-only crate. If the crate later grows a public `lib.rs`, consider moving them into a true Cargo integration-test target at that point.
