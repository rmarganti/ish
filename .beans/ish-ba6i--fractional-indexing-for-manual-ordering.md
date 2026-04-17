---
# ish-ba6i
title: Fractional indexing for manual ordering
status: completed
type: task
priority: normal
created_at: 2026-04-17T13:31:37Z
updated_at: 2026-04-17T18:32:53Z
parent: ish-4qx4
blocked_by:
    - ish-0iv6
---

## Description\n\nImplement base-62 fractional indexing for manual sort ordering of ishoos.\n\nReference: `beans/pkg/bean/fractional.go`\n\n## Requirements\n\n- [x] Base-62 alphabet: `0-9A-Za-z`\n- [x] `order_between(a, b)` — returns a key that sorts lexicographically between `a` and `b`\n- [x] Handle edge cases: `a=""` (before b), `b=""` (after a), both empty (midpoint "V")\n- [x] `midpoint()`, `increment_key()`, `decrement_key()` helpers\n\n## Verification\n\n```bash\ncargo test\n```\n\nUnit tests:\n- `order_between("", "") == "V"`\n- `a < order_between(a, b) < b` for various a, b\n- Repeated insertion between same bounds produces monotonically sortable keys

## Summary of Changes

- Added base-62 fractional ordering helpers in `src/model/ishoo.rs`: `midpoint()`, `order_between()`, `increment_key()`, and `decrement_key()`.
- Covered lexicographic edge cases where adjacent keys require prefix extension, including keys generated between neighboring values and after an existing key.
- Added unit tests for empty-bound behavior, valid between-key generation, invalid ranges, and repeated insertions producing monotonically sortable keys.

## Validation

- `cargo fmt --all -- --check`
- `cargo test`
- `cargo clippy -- -D warnings`
