---
# ish-zhk8
title: 'Core store: simple substring search'
status: todo
type: task
created_at: 2026-04-17T13:32:26Z
updated_at: 2026-04-17T13:32:26Z
parent: ish-6cqj
blocked_by:
    - ish-gqts
---

## Description\n\nImplement simple case-insensitive substring search over ishoos.\n\n## Requirements\n\n- [ ] `search(query)` — case-insensitive substring match across `title`, `slug`, and `body` fields\n- [ ] Return all matching ishoos (no ranking/scoring needed)\n- [ ] Used by the `list --search` CLI flag\n\n## Verification\n\n```bash\ncargo test\n```\n\nUnit tests: matches in title, body, slug; case-insensitive; no match returns empty.
