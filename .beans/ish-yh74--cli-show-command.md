---
# ish-yh74
title: 'CLI: show command'
status: todo
type: task
priority: normal
created_at: 2026-04-17T13:35:00Z
updated_at: 2026-04-17T18:23:35Z
parent: ish-gt2k
blocked_by:
    - ish-ffou
    - ish-qpqo
    - ish-vb4z
    - ish-bpxr
    - ish-2iuf
---

## Description\n\nImplement `ish show <id> [id...]` — display full ishoo details.\n\nReference: `beans/internal/commands/show.go`\n\n## Requirements\n\n- [ ] Accept one or more IDs\n- [ ] Flags: `--json`, `--raw` (raw markdown), `--body-only`, `--etag-only` (mutually exclusive)\n- [ ] Default styled output: header (ID, status badge, type badge, priority badge, tags, timestamps), separator, relationships (parent, blocking), separator, rendered markdown body\n- [ ] Multiple ishoos separated by `═` line\n- [ ] Short ID support (auto-prepend prefix)\n\n## Verification\n\n```bash\ncargo test\n```\n\nIntegration tests: show with each output mode, verify format.
