---
# ish-6z0n
title: 'CLI: delete command'
status: todo
type: task
created_at: 2026-04-17T13:35:00Z
updated_at: 2026-04-17T13:35:00Z
parent: ish-gt2k
blocked_by:
    - ish-sr0l
    - ish-h9yk
    - ish-qpqo
    - ish-vb4z
---

## Description\n\nImplement `ish delete <id> [id...]` (alias `rm`) — delete ishoos with confirmation.\n\nReference: `beans/internal/commands/delete.go`\n\n## Requirements\n\n- [ ] Accept one or more IDs\n- [ ] `-f/--force` — skip confirmation\n- [ ] `--json` — JSON output (implies force)\n- [ ] Before deleting, find incoming links and warn user\n- [ ] On confirmation, delete file and clean up all references in other ishoos\n- [ ] Interactive confirmation prompt: show title, path, incoming link count\n\n## Verification\n\n```bash\ncargo test\n```\n\nIntegration tests: delete with force, verify file removed and references cleaned.
