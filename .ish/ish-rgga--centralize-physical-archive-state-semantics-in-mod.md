---
# ish-rgga
title: Centralize physical archive-state semantics in model and store
status: completed
type: task
priority: high
tags:
- archive
- core
- json
created_at: 2026-04-30T19:00:46.415638Z
updated_at: 2026-04-30T19:08:30.000000Z
parent: ish-i6nu
blocking:
- ish-5pws
- ish-d7pz
- ish-64cd
---

## Context
This is the foundation for the archived/inactive-state feature. Today archive detection is duplicated with raw `path.starts_with("archive/")` checks, and blocker logic still treats archive-eligible statuses as the primary notion of inactivity.

Per the agreed plan, physical archive location under `.ish/archive/` becomes the source of truth for whether an ish is archived. Other work should build on a shared model helper instead of repeating path-prefix logic.

## Dependencies
- Parent feature will group the broader archive-state change.
- No prerequisite child work.

## Work
- Add a shared `Ish::is_archived()` helper in `src/model/ish/mod.rs`.
- Add `archived: bool` to `IshJson` and populate it from the helper.
- Replace direct `path.starts_with("archive/")` checks with the shared helper where practical, including existing TUI/store callers.
- Update store archive semantics so physically archived ishes never count as active blockers:
  - `Store::is_archived()` should delegate to the model helper
  - `has_active_status()` should reject physically archived ishes
  - `find_active_blockers()` should ignore physically archived blockers
- Keep archive-status config behavior (`completed` / `scrapped`) unchanged for archive eligibility, dimming, and inherited terminal-status semantics.

## Verification
- [x] Model tests cover `Ish::to_json()` with `archived: false` and `archived: true`.
- [x] Store tests prove archived blockers do not block active ishes while active blockers still do.
- [x] TUI/model coverage stays green without changing the TUI's active-only contract.
- [x] `mise exec -- ish check`
- [x] `mise run ci`

## Implementation notes
- Added `Ish::is_archived()` in `src/model/ish/mod.rs` and made `IshJson` expose a first-class `archived` boolean so later list/show/roadmap/check work can consume shared archive state instead of re-parsing paths.
- Updated `Store::is_archived()`, `archive_all_completed()`, `find_active_blockers()`, and `has_active_status()` to use the shared helper; physically archived ishes now never count as active blockers even if their status is still non-terminal.
- Replaced the TUI board model's private archive predicate with `ish.is_archived()`, keeping the existing active-only board behavior while removing one of the duplicated raw path-prefix checks.
- Added regression coverage in `src/model/ish/tests.rs` for archived JSON output and in `src/core/store/tests.rs` for archived blockers in both `blocked_by` and incoming `blocking` directions.

## Validation
- `mise exec -- cargo test model::ish -- --nocapture`
- `mise exec -- cargo test core::store -- --nocapture`
- `mise exec -- cargo test tui::model -- --nocapture`
- `mise exec -- cargo test output::tests:: -- --nocapture`
- `mise exec -- ish check`
- `mise run ci`

## Follow-up notes
- The next children (`ish-5pws`, `ish-d7pz`, `ish-64cd`) can now rely on `Ish::is_archived()` plus `IshJson.archived` instead of adding more direct `path.starts_with("archive/")` checks.
- `archive_all_completed()` still keys off configured archive-eligible statuses for deciding what to move; this task only changed how already-archived items are recognized after they live under `.ish/archive/`.
