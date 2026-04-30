# Progress

## 2026-04-30
- Chose `ish-rgga` as the best next ready ish because it is the foundation for the archived/inactive-state feature and blocks the remaining child tasks.
- Added `Ish::is_archived()` and surfaced archive state in JSON via `IshJson.archived`.
- Updated store blocker/archive helpers so physically archived ishes never count as active blockers, while archive-status config semantics remain unchanged for archive eligibility.
- Replaced the TUI board model's local archive-path check with the shared model helper.
- Added regression coverage for archived JSON output and archived blockers, then ran `mise exec -- ish check` and `mise run ci` successfully.
- Best next work on this feature: `ish-5pws` (`ish list` archive visibility + tree context), then `ish-d7pz` (roadmap/show output), then `ish-64cd` (`ish check` archive-state warnings).
