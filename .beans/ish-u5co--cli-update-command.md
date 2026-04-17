---
# ish-u5co
title: 'CLI: update command'
status: completed
type: task
priority: normal
created_at: 2026-04-17T13:35:00Z
updated_at: 2026-04-17T18:04:40Z
parent: ish-gt2k
blocked_by:
    - ish-sr0l
    - ish-qpqo
    - ish-vb4z
    - ish-fkhv
---

## Description

Implement `ish update <id>` (alias `u`) to update ishoo fields, body, tags, and relationships.

Reference: `beans/internal/commands/update.go`

## Requirements

- [x] Flags: `-s/--status`, `-t/--type`, `-p/--priority`, `--title`, `-d/--body`, `--body-file`, `--body-replace-old` + `--body-replace-new` (required together), `--body-append`
- [x] Relationship flags: `--parent`, `--remove-parent`, `--blocking` (repeatable), `--remove-blocking` (repeatable), `--blocked-by` (repeatable), `--remove-blocked-by` (repeatable)
- [x] Tag flags: `--tag` (repeatable, add), `--remove-tag` (repeatable)
- [x] `--if-match <etag>` for optimistic concurrency control
- [x] `--json` flag
- [x] Mutual exclusions: body/body-file vs body-replace-old, body/body-file vs body-append, parent vs remove-parent
- [x] `body-append` supports `-` for stdin
- [x] Auto-unarchive: archived ishoos are moved back to the main store before update
- [x] Error if no changes specified

## Verification

```bash
cargo fmt --all -- --check
cargo test
cargo clippy -- -D warnings
```

Integration tests cover field updates, file rename behavior, ETag conflicts, stdin body reading, relation removals, and auto-unarchive.

## Summary of Changes

Added the `ish update` / `ish u` CLI command with support for field updates, relationship and tag changes, ETag checks, JSON output, body replacement and append flows, and archived-item auto-unarchive before update.

Added command-level tests covering successful updates, conflict handling, no-op validation, stdin body reading, relation removals, file rename behavior, and auto-unarchive.
