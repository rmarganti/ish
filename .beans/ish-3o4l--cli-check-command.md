---
# ish-3o4l
title: 'CLI: check command'
status: todo
type: task
created_at: 2026-04-17T13:35:00Z
updated_at: 2026-04-17T13:35:00Z
parent: ish-gt2k
blocked_by:
    - ish-h9yk
    - ish-qpqo
    - ish-vb4z
---

## Description\n\nImplement `ish check` — validate configuration and link integrity.\n\nReference: `beans/internal/commands/check.go`\n\n## Requirements\n\n- [ ] Config checks: validate default_status, default_type, colors\n- [ ] Link checks: broken links, self-references, cycles (via `check_all_links()`)\n- [ ] `--fix` — auto-fix broken links and self-references (not cycles)\n- [ ] `--json` — JSON output with structured result\n- [ ] Exit code 1 if issues found\n- [ ] Human output: ✓/✗ for each check, summary at end\n\n## Verification\n\n```bash\ncargo test\n```\n\nIntegration tests: create ishoos with broken links, run check, verify detection. Run check --fix, verify fixes applied.
