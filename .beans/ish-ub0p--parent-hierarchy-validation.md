---
# ish-ub0p
title: Parent hierarchy validation
status: todo
type: task
created_at: 2026-04-17T13:33:29Z
updated_at: 2026-04-17T13:33:29Z
parent: ish-idzc
blocked_by:
    - ish-ffou
---

## Description\n\nImplement parent-child relationship validation enforcing the type hierarchy.\n\nReference: `beans/pkg/beancore/links.go` — `ValidParentTypes()`, `ValidateParent()`.\n\n## Requirements\n\n- [ ] `valid_parent_types(ishoo_type)` — returns allowed parent types:\n  - milestone: no parent allowed\n  - epic: [milestone]\n  - feature: [milestone, epic]\n  - task/bug: [milestone, epic, feature]\n- [ ] `validate_parent(ishoo, parent_id)` — look up parent, check type is in allowed list\n- [ ] Integrate validation into create and update flows\n\n## Verification\n\n```bash\ncargo test\n```\n\nUnit tests: valid and invalid parent assignments for each type combination.
