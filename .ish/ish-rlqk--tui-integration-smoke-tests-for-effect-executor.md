---
# ish-rlqk
title: 'TUI: integration smoke tests for effect executor'
status: todo
type: task
priority: high
tags:
- tui
created_at: 2026-04-25T03:20:55.843791Z
updated_at: 2026-04-25T03:21:17.824383Z
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
