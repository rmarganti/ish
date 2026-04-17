---
# ish-zhk8
title: 'Core store: simple substring search'
status: completed
type: task
priority: normal
created_at: 2026-04-17T13:32:26Z
updated_at: 2026-04-17T15:20:59Z
parent: ish-6cqj
blocked_by:
    - ish-gqts
---

## Description\n\nImplement simple case-insensitive substring search over ishoos.\n\n## Requirements\n\n- [x] `search(query)` — case-insensitive substring match across `title`, `slug`, and `body` fields\n- [x] Return all matching ishoos (no ranking/scoring needed)\n- [x] Used by the `list --search` CLI flag\n\n## Verification\n\n```bash\ncargo test\n```\n\nUnit tests: matches in title, body, slug; case-insensitive; no match returns empty.

## Summary of Changes

Added a reusable `src/core::search()` helper that performs case-insensitive substring matching across `Ishoo.title`, `Ishoo.slug`, and `Ishoo.body` while preserving input order.

Added focused unit tests for title, body, and slug matches, case-insensitive behavior, and empty-result behavior. The helper is marked `#[allow(dead_code)]` until the blocked `ish list --search` command is implemented, so the shared core behavior is available without breaking `clippy`.
