---
# ish-5cna
title: Implicit blocking and implicit status
status: completed
type: task
priority: normal
created_at: 2026-04-17T13:33:29Z
updated_at: 2026-04-17T16:10:09Z
parent: ish-idzc
blocked_by:
    - ish-ffou
---

## Description\n\nImplement transitive blocking (via parent chain) and implicit terminal status inheritance.\n\nReference: `beans/pkg/beancore/links.go` — `IsBlocked()`, `FindActiveBlockers()`, `FindAllBlockers()`, `ImplicitStatus()`, `walkParentChain()`.\n\n## Requirements\n\n- [x] `find_active_blockers(id)` — direct blockers (from blocked_by + incoming blocking) with non-resolved status\n- [x] `find_all_blockers(id)` — include blockers on ancestors via parent chain\n- [x] `is_blocked(id)` — true if any active blockers exist (direct or inherited)\n- [x] `is_explicitly_blocked(id)` — only direct blockers\n- [x] `is_implicitly_blocked(id)` — only ancestor blockers\n- [x] `implicit_status(id)` — walk parent chain, return first terminal status (completed/scrapped) + the ancestor ID\n- [x] `walk_parent_chain(id, visitor)` — DFS with cycle protection\n- [x] A blocker is "active" if its status is NOT completed or scrapped\n\n## Verification\n\n```bash\ncargo test\n```\n\nUnit tests: direct blocking, transitive blocking via parent, implicit status from grandparent, cycle in parent chain doesn't infinite loop.

## Summary of Changes

Added `Store` helpers for direct and inherited blocker discovery, blocked-state checks, and implicit terminal status inheritance via a cycle-safe parent-chain walk. Added unit coverage for direct blocking from both link directions, transitive parent blocking, inherited status from a grandparent, and parent-cycle protection.

## Verification Notes

- Ran `cargo fmt --all -- --check`
- Ran `cargo test`
- Ran `cargo clippy -- -D warnings`
