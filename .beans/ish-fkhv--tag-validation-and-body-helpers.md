---
# ish-fkhv
title: Tag validation and body helpers
status: todo
type: task
created_at: 2026-04-17T13:31:37Z
updated_at: 2026-04-17T13:31:37Z
parent: ish-4qx4
blocked_by:
    - ish-0iv6
---

## Description\n\nImplement tag validation/normalization and body content manipulation helpers.\n\nReference: `beans/pkg/bean/bean.go` (tags) and `beans/pkg/bean/content.go` (body helpers).\n\n## Requirements\n\nTags:\n- [ ] `validate_tag(tag)` — must match `^[a-z][a-z0-9]*(-[a-z0-9]+)*$`\n- [ ] `normalize_tag(tag)` — lowercase + trim\n- [ ] `has_tag()`, `add_tag()`, `remove_tag()` methods on Ishoo\n\nBody helpers:\n- [ ] `replace_once(text, old, new)` — error if old is empty, not found, or found multiple times\n- [ ] `unescape_body(s)` — interpret `\\n`, `\\t`, `\\\\` escape sequences\n- [ ] `append_with_separator(text, addition)` — join with blank line separator\n\n## Verification\n\n```bash\ncargo test\n```\n\nUnit tests for each function, including edge cases (empty strings, multiple occurrences, etc.)
