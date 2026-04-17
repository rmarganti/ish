---
# ish-h9yk
title: Cycle detection and link checking
status: completed
type: task
priority: normal
created_at: 2026-04-17T13:33:29Z
updated_at: 2026-04-17T16:05:14Z
parent: ish-idzc
blocked_by:
    - ish-ffou
---

## Description\n\nImplement cycle detection for link types and comprehensive link validation.\n\nReference: `beans/pkg/beancore/links.go` — `DetectCycle()`, `CheckAllLinks()`, `FixBrokenLinks()`, `FindIncomingLinks()`.\n\n## Requirements\n\n- [x] `detect_cycle(from_id, link_type, to_id)` — DFS to check if adding a link would create a cycle. Check blocking, blocked_by, parent link types\n- [x] `find_incoming_links(target_id)` — find all ishoos that link TO the given ID (parent, blocking, blocked_by)\n- [x] `check_all_links()` — validate all links, return `LinkCheckResult` with:\n  - `broken_links`: links to non-existent ishoos\n  - `self_links`: ishoos linking to themselves\n  - `cycles`: circular dependencies (per link type)\n- [x] `fix_broken_links()` — remove broken links and self-references, save affected files. Return count of fixes. Cannot auto-fix cycles.\n\n## Verification\n\n```bash\ncargo test\n```\n\nUnit tests: cycle detection (A→B→C→A), broken link detection, self-reference detection, fix removes bad links.

## Summary of Changes

Added link-validation primitives to `Store`: `LinkType`, `LinkRef`, `LinkCycle`, and `LinkCheckResult`, plus `detect_cycle`, `find_incoming_links`, `check_all_links`, and `fix_broken_links`.

Added store tests covering A->B->C->A cycle detection, incoming link discovery, broken/self-link reporting, and on-disk cleanup of invalid references. Also stabilized cwd-sensitive command tests in `src/main.rs` with a shared test lock so `cargo test` passes reliably under parallel execution.
