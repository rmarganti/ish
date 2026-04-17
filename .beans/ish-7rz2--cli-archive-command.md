---
# ish-7rz2
title: 'CLI: archive command'
status: todo
type: task
created_at: 2026-04-17T13:35:00Z
updated_at: 2026-04-17T13:35:00Z
parent: ish-gt2k
blocked_by:
    - ish-i6i7
    - ish-qpqo
    - ish-vb4z
---

## Description\n\nImplement `ish archive` — move all completed/scrapped ishoos to the archive directory.\n\nReference: `beans/internal/commands/archive.go`\n\n## Requirements\n\n- [ ] Find all ishoos with archive status (completed, scrapped)\n- [ ] Move each to `.ish/archive/`\n- [ ] `--json` flag\n- [ ] Output count of archived ishoos\n- [ ] No-op message if nothing to archive\n\n## Verification\n\n```bash\ncargo test\n```\n\nIntegration test: create completed ishoos, run archive, verify files moved.
