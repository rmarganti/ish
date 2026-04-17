---
# ish-ucno
title: 'CLI: roadmap command'
status: completed
type: task
priority: normal
created_at: 2026-04-17T13:35:00Z
updated_at: 2026-04-17T15:53:38Z
parent: ish-gt2k
blocked_by:
    - ish-ffou
    - ish-l4gd
---

## Description

Implement `ish roadmap` - generate a markdown roadmap from milestone/epic hierarchy.

Reference: `beans/internal/commands/roadmap.go` and `roadmap.tmpl`.

## Requirements

- [x] Build roadmap structure: milestones -> epics -> items, plus unscheduled group
- [x] Markdown output: headings for milestones, subheadings for epics, bullet lists for items with status badges
- [x] `--json` - structured JSON output
- [x] `--include-done` - include completed items
- [x] `--status` / `--no-status` - filter milestones by status
- [x] `--no-links` - don't render IDs as markdown links
- [x] `--link-prefix` - URL prefix for links
- [x] Items sorted by type then status

## Verification

```bash
cargo test
```

Unit tests: build roadmap from test data, verify markdown structure.

## Summary of Changes

- Added an `ish roadmap` subcommand with markdown and JSON output, milestone status filtering, include-done handling, and configurable link rendering.
- Implemented roadmap hierarchy building for milestones, epics, direct milestone items, and unscheduled work, with tests covering grouping, filtering, JSON structure, and command integration.
- Verified with `cargo fmt --all`, `cargo test`, and `cargo clippy -- -D warnings`.
