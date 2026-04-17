---
# ish-6ye6
title: 'CLI: create command'
status: todo
type: task
created_at: 2026-04-17T13:35:00Z
updated_at: 2026-04-17T13:35:00Z
parent: ish-gt2k
blocked_by:
    - ish-sr0l
    - ish-qpqo
    - ish-vb4z
---

## Description\n\nImplement `ish create [title]` — create a new ishoo.\n\nReference: `beans/internal/commands/create.go`\n\n## Requirements\n\n- [ ] Positional arg: title (default: "Untitled")\n- [ ] Flags: `-s/--status`, `-t/--type`, `-p/--priority`, `-d/--body`, `--body-file`, `--tag` (repeatable), `--parent`, `--blocking` (repeatable), `--blocked-by` (repeatable), `--prefix`, `--json`\n- [ ] `--body` and `--body-file` mutually exclusive; `--body -` reads from stdin\n- [ ] Validate status, type, priority against config\n- [ ] Apply config defaults for status and type if not specified\n- [ ] Output: styled "Created {id} {path}" or JSON\n\n## Verification\n\n```bash\ncargo test\n```\n\nIntegration tests: create with various flag combinations, verify file on disk.
