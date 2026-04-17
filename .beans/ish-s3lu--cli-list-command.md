---
# ish-s3lu
title: 'CLI: list command'
status: todo
type: task
created_at: 2026-04-17T13:35:00Z
updated_at: 2026-04-17T13:35:00Z
parent: ish-gt2k
blocked_by:
    - ish-ffou
    - ish-zhk8
    - ish-l4gd
    - ish-k60j
    - ish-qpqo
    - ish-vb4z
    - ish-5cna
---

## Description\n\nImplement `ish list` (alias `ls`) — list and filter ishoos.\n\nReference: `beans/internal/commands/list.go`\n\n## Requirements\n\n- [ ] Filter flags: `-s/--status` (repeatable), `--no-status` (repeatable), `-t/--type` (repeatable), `--no-type` (repeatable), `-p/--priority` (repeatable), `--no-priority` (repeatable), `--tag` (repeatable, OR logic), `--no-tag` (repeatable)\n- [ ] Parent/blocking filters: `--has-parent`, `--no-parent`, `--parent <id>`, `--has-blocking`, `--no-blocking`, `--is-blocked`\n- [ ] `--ready` — not blocked, excludes in-progress/completed/scrapped/draft, excludes implicit terminal status\n- [ ] `--search/-S <query>` — substring search\n- [ ] `--sort <mode>` — created, updated, status, priority, id (default: status+priority+type+title)\n- [ ] `--quiet/-q` — print only IDs\n- [ ] `--full` — include body in JSON output\n- [ ] `--json` — JSON output (flat list)\n- [ ] Default output: tree view with styled rendering\n\n## Verification\n\n```bash\ncargo test\n```\n\nIntegration tests: create several ishoos, verify filter combinations produce correct results.
