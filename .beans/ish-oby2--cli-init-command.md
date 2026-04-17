---
# ish-oby2
title: 'CLI: init command'
status: completed
type: task
priority: normal
created_at: 2026-04-17T13:35:00Z
updated_at: 2026-04-17T16:22:57Z
parent: ish-gt2k
blocked_by:
    - ish-614c
    - ish-b8m7
    - ish-qpqo
---

## Description\n\nImplement `ish init` — initialize a new ish project in the current directory.\n\nReference: `beans/internal/commands/init.go`\n\n## Requirements\n\n- [x] Create `.ish/` directory\n- [x] Write `.ish/.gitignore` (exclude `.conversations/`)\n- [x] Create `.ish.yml` config file with defaults (prefix = `{dirname}-`, project.name = dirname)\n- [x] `--json` flag for JSON output\n- [x] Idempotent: don't overwrite existing config\n\n## Verification\n\n```bash\ncargo test\n```\n\nIntegration test: init in temp dir, verify directory and config created.

## Summary of Changes

Added the `ish init` CLI command to create `.ish/`, write `.ish/.gitignore`, and initialize `.ish.yml` with a directory-derived project name and `{dirname}-` prefix.

The command now returns a success message in both plain text and `--json` modes, preserves any existing config on re-run, and is covered by tests for initialization, JSON output, and idempotent behavior.

## Verification Notes

- Ran `cargo fmt --all`
- Ran `cargo test`
- Ran `cargo clippy -- -D warnings`
