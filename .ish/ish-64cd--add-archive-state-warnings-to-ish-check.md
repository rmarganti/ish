---
# ish-64cd
title: Add archive-state warnings to ish check
status: completed
type: task
priority: normal
tags:
- archive
- check
- validation
created_at: 2026-04-30T19:00:46.440310Z
updated_at: 2026-04-30T19:20:03.555564Z
parent: ish-i6nu
blocked_by:
- ish-rgga
---

## Context
The new archive semantics intentionally do not propagate inactivity through parent/child links, but that means the workspace can contain suspicious mixed active/archived relationships that deserve visibility. `ish check` should keep validating archived ishes and add warnings for these cases without turning them into normal broken-link failures.

The planned warning categories are:
1. active child with archived parent
2. active ish references archived ish
3. archived ish references active ish

References include parent links plus `blocking` / `blocked_by` relationships.

## Dependencies
- Blocked by the shared archive helper / core semantics task.

## Work
- Define structured archive-warning result types suitable for human and JSON output.
- Add a store-level helper that walks loaded ishes, detects mixed active/archived relationships, and returns stable, deduplicated warnings.
- Thread the new warnings through `src/commands/check/mod.rs` and `src/commands/check/render.rs`.
- Render warnings as warnings in human output and as structured `archive_warnings` data in JSON output.
- Keep exit-code semantics aligned with current warning policy unless existing tests/product policy require otherwise.
- Add focused tests for all three warning categories plus JSON serialization.

## Verification
- `ish check` human output shows a dedicated archive-warning section when mixed relationships exist.
- `ish check --json` includes structured `archive_warnings` entries with stable kinds/IDs.
- Warning-only cases do not regress normal broken-link/cycle validation behavior.
- `mise exec -- ish check`
- `mise run ci`

## Implementation notes
- Added `Store::find_archive_warnings()` plus structured `ArchiveWarning` / `ArchiveWarningKind` records so `ish check` can detect mixed active/archived parent, `blocking`, and `blocked_by` relationships with stable sorting.
- Threaded archive warnings through `src/commands/check/mod.rs` and `src/commands/check/render.rs`; human output now shows a dedicated warning section while JSON exposes `checks.archive_warnings` and `summary.archive_warning_count`.
- Warning-only archive-state cases remain exit-code clean; only config/link issues count as check failures.

## Verification
- `mise exec -- ish check`
- `mise run ci`
