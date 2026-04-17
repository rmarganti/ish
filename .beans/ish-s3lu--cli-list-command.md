---
# ish-s3lu
title: 'CLI: list command'
status: completed
type: task
priority: normal
created_at: 2026-04-17T13:35:00Z
updated_at: 2026-04-17T17:59:45Z
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

## Description\n\nImplement `ish list` (alias `ls`) — list and filter ishoos.\n\nReference: `beans/internal/commands/list.go`\n\n## Requirements\n\n- [x] Filter flags: `-s/--status` (repeatable), `--no-status` (repeatable), `-t/--type` (repeatable), `--no-type` (repeatable), `-p/--priority` (repeatable), `--no-priority` (repeatable), `--tag` (repeatable, OR logic), `--no-tag` (repeatable)\n- [x] Parent/blocking filters: `--has-parent`, `--no-parent`, `--parent <id>`, `--has-blocking`, `--no-blocking`, `--is-blocked`\n- [x] `--ready` — not blocked, excludes in-progress/completed/scrapped/draft, excludes implicit terminal status\n- [x] `--search/-S <query>` — substring search\n- [x] `--sort <mode>` — created, updated, status, priority, id (default: status+priority+type+title)\n- [x] `--quiet/-q` — print only IDs\n- [x] `--full` — include body in JSON output\n- [x] `--json` — JSON output (flat list)\n- [x] Default output: tree view with styled rendering\n\n## Verification\n\n```bash\ncargo test\n```\n\nIntegration tests: create several ishoos, verify filter combinations produce correct results.

## Summary of Changes

Implemented `ish list` / `ish ls` with repeatable status, type, priority, tag, parent, blocking, readiness, and substring-search filters plus `created`, `updated`, `status`, `priority`, and `id` sort modes. Wired human-readable tree output through the shared styling/tree helpers and added flat JSON list output with `--full` controlling whether `body` is included.

Added focused command tests for alias parsing, JSON filtering, `--full`, `--ready`, and tree rendering so the new CLI behavior is regression-covered.

## Notes for Future Workers

`ish list --json` intentionally returns a flat `ishoos` array, while human-readable mode preserves ancestor context through tree rendering for filtered descendants.

## Verification Notes

- Ran `cargo fmt --all`
- Ran `cargo test`
- Ran `cargo clippy -- -D warnings`
