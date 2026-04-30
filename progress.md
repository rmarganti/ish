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
