---
# ish-4b45
title: 'CLI: prime command'
status: todo
type: task
priority: normal
created_at: 2026-04-17T13:35:00Z
updated_at: 2026-04-17T18:23:35Z
parent: ish-gt2k
blocked_by:
    - ish-614c
---

## Description\n\nImplement `ish prime` — output agent prompt instructions.\n\nReference: `beans/internal/commands/prime.go` and `prompt.tmpl`.\n\n## Requirements\n\n- [ ] Template-based output that teaches AI agents how to use the `ish` CLI\n- [ ] Include: CLI command reference, types, statuses, priorities, body modification guide, concurrency control, relationship model\n- [ ] Use ish/ishoo terminology throughout (not beans)\n- [ ] Silently exit if no `.ish.yml` found (don't error)\n- [ ] Populate template with hardcoded types, statuses, priorities from config\n\n## Verification\n\nManual: run `ish prime` in a project, verify output is a complete agent guide.
