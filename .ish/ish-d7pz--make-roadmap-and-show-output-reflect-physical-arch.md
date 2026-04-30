---
# ish-d7pz
title: Make roadmap and show output reflect physical archive state
status: completed
type: task
priority: normal
tags:
- archive
- roadmap
- output
created_at: 2026-04-30T19:00:46.431848Z
updated_at: 2026-04-30T19:14:43.656507Z
parent: ish-i6nu
blocked_by:
- ish-rgga
---

## Context
Once physical archive state is modeled centrally, user-facing output should expose it consistently. Two concrete behavior changes are already agreed:
- `ish roadmap` hides physically archived ishes by default, even with `--include-done`
- human `ish show` output should print an explicit `Archived: yes/no` line so archive state is obvious without inspecting the path

JSON output picks up the new `archived` field from the shared model work, so this task should focus on command-level behavior and regression coverage.

## Dependencies
- Blocked by the shared archive helper / core semantics task.

## Work
- Update `src/roadmap/mod.rs` so physically archived ishes are filtered out before done-status filtering.
- Preserve the existing meaning of `--include-done`: include completed active items, but still exclude physically archived items.
- Update `src/commands/show.rs` human rendering to include an explicit `Archived: yes/no` line near the path/metadata block.
- Add regression coverage for roadmap archived filtering and the new show output line.
- Confirm transitive JSON snapshots/expectations that now include `archived` stay accurate.

## Verification
- Roadmap tests prove archived items are excluded by default and still excluded when `include_done: true`.
- Show-command tests prove human output includes `Archived: yes` / `Archived: no` as appropriate.
- Any affected JSON output tests are updated to include the new `archived` field.
- `mise exec -- ish check`
- `mise run ci`


## Implementation notes
- Updated `src/roadmap/mod.rs` so roadmap visibility now filters out physically archived ishes before applying the existing done-status logic, which keeps `--include-done` limited to active completed items.
- Added roadmap regression coverage in `src/roadmap/tests.rs` for archived milestones/epics/features to prove archived entries stay hidden even when `include_done: true` is enabled.
- Updated `src/commands/show.rs` human output to print `Archived: yes/no` directly under the path, and extended show-command tests to cover both active and archived items plus the transitive JSON `archived` field expectation.

## Validation
- `mise exec -- cargo test roadmap -- --nocapture`
- `mise exec -- cargo test show_command -- --nocapture`
- `mise exec -- ish check`
- `mise run ci`

## Follow-up notes
- The remaining archive-state feature work is `ish-64cd` (`ish check` warnings for mixed active/archived relationships); roadmap and show now consume the shared `Ish::is_archived()` semantics consistently.
