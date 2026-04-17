---
# ish-6z0n
title: 'CLI: delete command'
status: completed
type: task
priority: normal
created_at: 2026-04-17T13:35:00Z
updated_at: 2026-04-17T17:52:29Z
parent: ish-gt2k
blocked_by:
    - ish-sr0l
    - ish-h9yk
    - ish-qpqo
    - ish-vb4z
---

## Description\n\nImplement `ish delete <id> [id...]` (alias `rm`) — delete ishoos with confirmation.\n\nReference: `beans/internal/commands/delete.go`\n\n## Requirements\n\n- [x] Accept one or more IDs\n- [x] `-f/--force` — skip confirmation\n- [x] `--json` — JSON output (implies force)\n- [x] Before deleting, find incoming links and warn user\n- [x] On confirmation, delete file and clean up all references in other ishoos\n- [x] Interactive confirmation prompt: show title, path, incoming link count\n\n## Verification\n\n```bash\ncargo test\n```\n\nIntegration tests: delete with force, verify file removed and references cleaned.


## Summary of Changes

Added the `ish delete` / `ish rm` command with multi-ID support, confirmation prompts that show title/path/incoming link counts, forced and JSON deletion flows, and cleanup of remaining references after deletion.

Verified the behavior with command-level tests for confirmation, cancellation, forced deletion, JSON output, and alias parsing, plus the full cargo feedback loop.
