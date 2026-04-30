---
# ish-5pws
title: Add archive visibility modes to ish list and tree rendering
status: todo
type: task
priority: high
tags:
- archive
- cli
- list
created_at: 2026-04-30T19:00:46.423664Z
updated_at: 2026-04-30T19:00:46.423664Z
parent: ish-i6nu
blocked_by:
- ish-rgga
---

## Context
Default `ish list` should hide physically archived ishes, while `ish list --archived` should show archived-only and `ish list --all` should show active + archived. These visibility rules must be enforced before normal filters so `--status`, `--search`, and similar flags do not pierce archive hiding.

The list tree renderer also needs archive-aware context rules:
- default mode: no archived parents as context
- `--archived`: archived-only items with no active context
- `--all`: full hierarchy, including archived ancestors

## Dependencies
- Blocked by the shared archive helper / core semantics task.

## Work
- Extend `ListArgs` in `src/cli/mod.rs` with mutually exclusive `--archived` and `--all` flags.
- Add parser coverage for valid `--archived`, valid `--all`, and invalid combined use.
- Introduce an internal archive-visibility mode for list behavior instead of scattering booleans.
- Apply archive visibility at the top of `match_filters()` in `src/commands/list/filters.rs`.
- Update `is_ready()` so physically archived ishes never count as ready, including under `--all`.
- Change tree rendering in `src/commands/list/mod.rs` / `src/commands/list/render.rs` to use a visibility-scoped tree universe.
- Add focused list tests for default visibility, archived-only, all-items mode, filter non-piercing behavior, ready filtering, and tree-context behavior in each mode.

## Verification
- `ish list` hides archived ishes by default.
- `ish list --archived` shows only archived ishes.
- `ish list --all` shows both active and archived ishes.
- Archived items do not appear under `--ready`, even when `--all` is used.
- Tree rendering follows the agreed context rules in all three visibility modes.
- `mise exec -- ish check`
- `mise run ci`
