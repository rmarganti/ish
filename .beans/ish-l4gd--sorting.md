---
# ish-l4gd
title: Sorting
status: completed
type: task
priority: normal
created_at: 2026-04-17T13:31:37Z
updated_at: 2026-04-17T15:41:54Z
parent: ish-4qx4
blocked_by:
    - ish-0iv6
---

## Description\n\nImplement multi-level sorting for ishoos.\n\nReference: `beans/pkg/bean/sort.go`\n\n## Requirements\n\n- [x] `sort_by_status_priority_and_type(ishoos, status_names, priority_names, type_names)` — the default sort\n- [x] Sort order: status → manual order (fractional index, ishoos with order come first) → priority (empty = normal) → type → title (case-insensitive)\n- [x] Alternative sort modes: `created`, `updated`, `status`, `priority`, `id`\n- [x] Unrecognized values sort last within their category\n\n## Verification\n\n```bash\ncargo test\n```\n\nUnit tests with ishoos in various states verifying correct ordering.

## Summary of Changes

Added reusable sorting helpers in `src/core/mod.rs` for the default multi-level ordering plus explicit `created`, `updated`, `status`, `priority`, and `id` sort modes. Added unit coverage for manual-order precedence, case-insensitive title tie-breaking, and unrecognized status/priority/type values sorting last.

## Verification Notes

Validated with `cargo fmt --all -- --check`, `cargo test`, and `cargo clippy -- -D warnings`.
