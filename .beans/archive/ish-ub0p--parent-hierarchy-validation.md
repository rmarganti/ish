---
# ish-ub0p
title: Parent hierarchy validation
status: completed
type: task
priority: normal
created_at: 2026-04-17T13:33:29Z
updated_at: 2026-04-17T16:15:20Z
parent: ish-idzc
blocked_by:
    - ish-ffou
---

## Description\n\nImplement parent-child relationship validation enforcing the type hierarchy.\n\nReference: `beans/pkg/beancore/links.go` — `ValidParentTypes()`, `ValidateParent()`.\n\n## Requirements\n\n- [x] `valid_parent_types(ishoo_type)` — returns allowed parent types:\n  - milestone: no parent allowed\n  - epic: [milestone]\n  - feature: [milestone, epic]\n  - task/bug: [milestone, epic, feature]\n- [x] `validate_parent(ishoo, parent_id)` — look up parent, check type is in allowed list\n- [x] Integrate validation into create and update flows\n\n## Verification\n\n```bash\ncargo test\n```\n\nUnit tests: valid and invalid parent assignments for each type combination.

## Summary of Changes

Added store-level parent hierarchy validation with `valid_parent_types()` and `validate_parent()` so create/update operations now reject missing parents, disallowed parent types, and parent assignments on milestones. Expanded `src/core/store.rs` tests to cover the hierarchy matrix, invalid create/update cases, and updated the existing CRUD tests to use real parent fixtures.

## Verification Notes

- Ran `cargo fmt --all -- --check`
- Ran `cargo test`
- Ran `cargo clippy -- -D warnings`
