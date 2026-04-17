---
# ish-h9yk
title: Cycle detection and link checking
status: todo
type: task
created_at: 2026-04-17T13:33:29Z
updated_at: 2026-04-17T13:33:29Z
parent: ish-idzc
blocked_by:
    - ish-ffou
---

## Description\n\nImplement cycle detection for link types and comprehensive link validation.\n\nReference: `beans/pkg/beancore/links.go` — `DetectCycle()`, `CheckAllLinks()`, `FixBrokenLinks()`, `FindIncomingLinks()`.\n\n## Requirements\n\n- [ ] `detect_cycle(from_id, link_type, to_id)` — DFS to check if adding a link would create a cycle. Check blocking, blocked_by, parent link types\n- [ ] `find_incoming_links(target_id)` — find all ishoos that link TO the given ID (parent, blocking, blocked_by)\n- [ ] `check_all_links()` — validate all links, return `LinkCheckResult` with:\n  - `broken_links`: links to non-existent ishoos\n  - `self_links`: ishoos linking to themselves\n  - `cycles`: circular dependencies (per link type)\n- [ ] `fix_broken_links()` — remove broken links and self-references, save affected files. Return count of fixes. Cannot auto-fix cycles.\n\n## Verification\n\n```bash\ncargo test\n```\n\nUnit tests: cycle detection (A→B→C→A), broken link detection, self-reference detection, fix removes bad links.
