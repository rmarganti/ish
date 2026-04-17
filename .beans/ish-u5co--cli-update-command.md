---
# ish-u5co
title: 'CLI: update command'
status: todo
type: task
created_at: 2026-04-17T13:35:00Z
updated_at: 2026-04-17T13:35:00Z
parent: ish-gt2k
blocked_by:
    - ish-sr0l
    - ish-qpqo
    - ish-vb4z
    - ish-fkhv
---

## Description\n\nImplement `ish update <id>` (alias `u`) — update ishoo fields, body, and relationships.\n\nReference: `beans/internal/commands/update.go`\n\n## Requirements\n\n- [ ] Flags: `-s/--status`, `-t/--type`, `-p/--priority`, `--title`, `-d/--body`, `--body-file`, `--body-replace-old` + `--body-replace-new` (required together), `--body-append`\n- [ ] Relationship flags: `--parent`, `--remove-parent`, `--blocking` (repeatable), `--remove-blocking` (repeatable), `--blocked-by` (repeatable), `--remove-blocked-by` (repeatable)\n- [ ] Tag flags: `--tag` (repeatable, add), `--remove-tag` (repeatable)\n- [ ] `--if-match <etag>` — optimistic concurrency control\n- [ ] `--json` flag\n- [ ] Mutual exclusions: body/body-file vs body-replace-old, body/body-file vs body-append, parent vs remove-parent\n- [ ] body-append supports `-` for stdin\n- [ ] Auto-unarchive: if ishoo is archived and being updated, move it back to main dir\n- [ ] Error if no changes specified\n\n## Verification\n\n```bash\ncargo test\n```\n\nIntegration tests: update each field type, verify file changes, ETag conflict, auto-unarchive.
