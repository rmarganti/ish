---
# ish-ucno
title: 'CLI: roadmap command'
status: todo
type: task
created_at: 2026-04-17T13:35:00Z
updated_at: 2026-04-17T13:35:00Z
parent: ish-gt2k
blocked_by:
    - ish-ffou
    - ish-l4gd
---

## Description\n\nImplement `ish roadmap` — generate a markdown roadmap from milestone/epic hierarchy.\n\nReference: `beans/internal/commands/roadmap.go` and `roadmap.tmpl`.\n\n## Requirements\n\n- [ ] Build roadmap structure: milestones → epics → items, plus unscheduled group\n- [ ] Markdown output: headings for milestones, subheadings for epics, bullet lists for items with status badges\n- [ ] `--json` — structured JSON output\n- [ ] `--include-done` — include completed items\n- [ ] `--status` / `--no-status` — filter milestones by status\n- [ ] `--no-links` — don't render IDs as markdown links\n- [ ] `--link-prefix` — URL prefix for links\n- [ ] Items sorted by type then status\n\n## Verification\n\n```bash\ncargo test\n```\n\nUnit tests: build roadmap from test data, verify markdown structure.
