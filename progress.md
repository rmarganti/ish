# Progress

## 2026-04-30
- Chose `ish-rgga` as the best next ready ish because it is the foundation for the archived/inactive-state feature and blocks the remaining child tasks.
- Added `Ish::is_archived()` and surfaced archive state in JSON via `IshJson.archived`.
- Updated store blocker/archive helpers so physically archived ishes never count as active blockers, while archive-status config semantics remain unchanged for archive eligibility.
- Replaced the TUI board model's local archive-path check with the shared model helper.
- Added regression coverage for archived JSON output and archived blockers, then ran `mise exec -- ish check` and `mise run ci` successfully.
- Best next work on this feature: `ish-5pws` (`ish list` archive visibility + tree context), then `ish-d7pz` (roadmap/show output), then `ish-64cd` (`ish check` archive-state warnings).
- Chose `ish-5pws` next because it was the highest-priority ready child and the strongest unblocker for user-facing archive semantics: `ish list` is the main discovery surface, and landing its visibility rules clarifies the inactive-state model before roadmap/show/check follow-ons.
- Added `ish list --archived` and `ish list --all`, enforced archive visibility before normal list filters, and scoped tree context to the same active-only / archived-only / full-universe mode so archived parents no longer leak into default trees.
- Tightened `ish list --ready` so physically archived ishes stay hidden even under `--all`, and added coverage for visibility modes, non-piercing completed filtering, ready filtering, and mixed active/archived tree context cases.
- `mise exec -- ish check` and `mise run ci` both pass after the list/archive visibility changes.
- Best next work is now `ish-d7pz` (roadmap + show archive-state output), then `ish-64cd` (`ish check` archive-state warnings).
- Chose `ish-d7pz` over the parent umbrella and `ish-64cd` because it is the strongest remaining user-facing slice after list visibility: it finishes archive-aware discovery/output semantics for roadmap and show without entangling the larger validation-warning plumbing.
- Updated `ish roadmap` to exclude physically archived ishes before done-status filtering, so `--include-done` now includes completed active items but still suppresses archived ones.
- Updated human `ish show` output to print `Archived: yes/no` directly under `Path:`, and expanded show tests to cover both active and archived items plus the transitive JSON `archived` field expectation.
- Added roadmap regression coverage proving archived epics/features stay hidden even when `include_done: true` is set.
- `mise exec -- ish check` and `mise run ci` both pass after the roadmap/show archive-state changes.
- Best next work is now `ish-64cd` (`ish check` archive-state warnings), which is the last remaining child for the archived/inactive-state feature.
- Chose `ish-64cd` because it was the strongest remaining ready task: it finishes the archived/inactive-state feature by surfacing suspicious mixed active/archived relationships without changing blocker or visibility semantics again.
- Added structured `ArchiveWarning` / `ArchiveWarningKind` detection in `Store::find_archive_warnings()` covering archived-parent, active→archived, and archived→active relationships across `parent`, `blocking`, and `blocked_by` links.
- Threaded archive warnings through `ish check` human and JSON output; human output now shows a dedicated warning section, JSON now exposes `checks.archive_warnings` plus `summary.archive_warning_count`, and warning-only cases still exit successfully.
- Added regression coverage for store-level warning detection plus human/JSON `ish check` behavior, then ran `mise exec -- ish check` and `mise run ci` successfully.
- All archived/inactive-state child tasks are now complete; the parent feature `ish-i6nu` is ready for closeout/archive when desired.
- Closed out `ish-i6nu` after confirming the full archived/inactive-state feature passes `mise exec -- ish check` and `mise run ci`; future archive follow-ons should reuse `Ish::is_archived()` and `archive_warnings` instead of reviving path-prefix or status-based ad hoc checks.
