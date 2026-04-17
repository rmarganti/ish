---
# ish-ba6i
title: Fractional indexing for manual ordering
status: todo
type: task
priority: normal
created_at: 2026-04-17T13:31:37Z
updated_at: 2026-04-17T18:23:35Z
parent: ish-4qx4
blocked_by:
    - ish-0iv6
---

## Description\n\nImplement base-62 fractional indexing for manual sort ordering of ishoos.\n\nReference: `beans/pkg/bean/fractional.go`\n\n## Requirements\n\n- [ ] Base-62 alphabet: `0-9A-Za-z`\n- [ ] `order_between(a, b)` — returns a key that sorts lexicographically between `a` and `b`\n- [ ] Handle edge cases: `a=""` (before b), `b=""` (after a), both empty (midpoint "V")\n- [ ] `midpoint()`, `increment_key()`, `decrement_key()` helpers\n\n## Verification\n\n```bash\ncargo test\n```\n\nUnit tests:\n- `order_between("", "") == "V"`\n- `a < order_between(a, b) < b` for various a, b\n- Repeated insertion between same bounds produces monotonically sortable keys
