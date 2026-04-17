---
# ish-5cna
title: Implicit blocking and implicit status
status: todo
type: task
created_at: 2026-04-17T13:33:29Z
updated_at: 2026-04-17T13:33:29Z
parent: ish-idzc
blocked_by:
    - ish-ffou
---

## Description\n\nImplement transitive blocking (via parent chain) and implicit terminal status inheritance.\n\nReference: `beans/pkg/beancore/links.go` — `IsBlocked()`, `FindActiveBlockers()`, `FindAllBlockers()`, `ImplicitStatus()`, `walkParentChain()`.\n\n## Requirements\n\n- [ ] `find_active_blockers(id)` — direct blockers (from blocked_by + incoming blocking) with non-resolved status\n- [ ] `find_all_blockers(id)` — include blockers on ancestors via parent chain\n- [ ] `is_blocked(id)` — true if any active blockers exist (direct or inherited)\n- [ ] `is_explicitly_blocked(id)` — only direct blockers\n- [ ] `is_implicitly_blocked(id)` — only ancestor blockers\n- [ ] `implicit_status(id)` — walk parent chain, return first terminal status (completed/scrapped) + the ancestor ID\n- [ ] `walk_parent_chain(id, visitor)` — DFS with cycle protection\n- [ ] A blocker is "active" if its status is NOT completed or scrapped\n\n## Verification\n\n```bash\ncargo test\n```\n\nUnit tests: direct blocking, transitive blocking via parent, implicit status from grandparent, cycle in parent chain doesn't infinite loop.
