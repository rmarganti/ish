---
# ish-i6nu
title: Archived ishes use physical inactive-state semantics
status: completed
type: feature
priority: high
tags:
- archive
- cli
- validation
created_at: 2026-04-30T19:00:46.406397Z
updated_at: 2026-04-30T19:21:36.182353Z
---

## Context
Archived/inactive behavior currently leaks across multiple code paths and is still tied in places to archive-eligible statuses (`completed` / `scrapped`) rather than the physical archive location under `.ish/archive/`.

The agreed product semantics are:
- physically archived ishes are hidden from normal discovery views
- archived ishes only appear via explicit archive visibility flags or explicit ID-based commands
- archived ishes never count as active blockers
- archived parents do not implicitly inactivate descendants
- roadmap and TUI stay active-only by default
- human and JSON output should make archive state explicit
- `ish check` should warn about suspicious active/archived mixed relationships

The implementation plan for this work lives at `.local/plans/1777575288-archived-inactive-state/plan.md`.

## Work
- Establish physical archive location as the single source of truth for archive state.
- Apply that semantics consistently across list, roadmap, blocker logic, show/output serialization, and validation.
- Land the work as a small set of independently completable child ishes with explicit dependency edges where ordering matters.

## Child breakdown
- Core archive predicate + store/json plumbing
- `ish list` archive visibility modes and tree-context behavior
- roadmap/show output changes
- `ish check` archive-state warnings

## Verification
- All child ishes are completed with focused tests.
- `mise exec -- ish check` reports a coherent workspace.
- `mise run ci` passes after the full feature lands.

## Implementation notes
- Closed out the archived/inactive-state feature after all four child tasks landed: `ish-rgga` (shared archive predicate + JSON/store plumbing), `ish-5pws` (`ish list` archive visibility + tree context), `ish-d7pz` (roadmap/show archive-state output), and `ish-64cd` (`ish check` archive-state warnings).
- The feature now uses physical archive location (`.ish/archive/`) as the single source of truth for inactive/archive semantics across list, roadmap, show/JSON, blocker readiness, TUI board modeling, and `ish check` warnings.
- Archive-eligible statuses (`completed` / `scrapped`) keep their existing archive-command / done-status meaning, but no longer implicitly stand in for physical archived state.

## Validation
- `mise exec -- ish check`
- `mise run ci`

## Follow-up notes
- No remaining child work is open for this feature. Future archive-state follow-ons should build on `Ish::is_archived()` and the structured `archive_warnings` output rather than reintroducing path-prefix checks or status-based archive semantics.
