---
# ish-l4gd
title: Sorting
status: todo
type: task
created_at: 2026-04-17T13:31:37Z
updated_at: 2026-04-17T13:31:37Z
parent: ish-4qx4
blocked_by:
    - ish-0iv6
---

## Description\n\nImplement multi-level sorting for ishoos.\n\nReference: `beans/pkg/bean/sort.go`\n\n## Requirements\n\n- [ ] `sort_by_status_priority_and_type(ishoos, status_names, priority_names, type_names)` — the default sort\n- [ ] Sort order: status → manual order (fractional index, ishoos with order come first) → priority (empty = normal) → type → title (case-insensitive)\n- [ ] Alternative sort modes: `created`, `updated`, `status`, `priority`, `id`\n- [ ] Unrecognized values sort last within their category\n\n## Verification\n\n```bash\ncargo test\n```\n\nUnit tests with ishoos in various states verifying correct ordering.
