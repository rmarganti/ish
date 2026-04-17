---
# ish-oby2
title: 'CLI: init command'
status: todo
type: task
created_at: 2026-04-17T13:35:00Z
updated_at: 2026-04-17T13:35:00Z
parent: ish-gt2k
blocked_by:
    - ish-614c
    - ish-b8m7
    - ish-qpqo
---

## Description\n\nImplement `ish init` — initialize a new ish project in the current directory.\n\nReference: `beans/internal/commands/init.go`\n\n## Requirements\n\n- [ ] Create `.ish/` directory\n- [ ] Write `.ish/.gitignore` (exclude `.conversations/`)\n- [ ] Create `.ish.yml` config file with defaults (prefix = `{dirname}-`, project.name = dirname)\n- [ ] `--json` flag for JSON output\n- [ ] Idempotent: don't overwrite existing config\n\n## Verification\n\n```bash\ncargo test\n```\n\nIntegration test: init in temp dir, verify directory and config created.
