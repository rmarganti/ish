---
# ish-7rz2
title: 'CLI: archive command'
status: completed
type: task
priority: normal
created_at: 2026-04-17T13:35:00Z
updated_at: 2026-04-17T16:29:07Z
parent: ish-gt2k
blocked_by:
    - ish-i6i7
    - ish-qpqo
    - ish-vb4z
---

## Description\n\nImplement `ish archive` — move all completed/scrapped ishoos to the archive directory.\n\nReference: `beans/internal/commands/archive.go`\n\n## Requirements\n\n- [x] Find all ishoos with archive status (completed, scrapped)\n- [x] Move each to `.ish/archive/`\n- [x] `--json` flag\n- [x] Output count of archived ishoos\n- [x] No-op message if nothing to archive\n\n## Verification\n\n```bash\ncargo test\n```\n\nIntegration test: create completed ishoos, run archive, verify files moved.

## Summary of Changes

- Added the `ish archive` CLI command in `src/main.rs`, wired through clap and backed by the existing store-level `archive_all_completed()` operation.
- Added a shared current-directory store loader so `archive` and `roadmap` resolve config and store paths consistently.
- Added command tests covering successful archiving, the no-op case, and JSON output with the archived count.

## Verification Notes

- `cargo fmt --all -- --check`
- `cargo test`
- `cargo clippy -- -D warnings`
