---
# ish-b8m7
title: Config discovery (walk directory tree)
status: completed
type: task
priority: normal
created_at: 2026-04-17T13:31:55Z
updated_at: 2026-04-17T15:09:24Z
parent: ish-ksgz
blocked_by:
    - ish-0iv6
---

## Description

Implement upward directory-tree search to find the nearest `.ish.yml` config file.

Reference: `beans/pkg/config/config.go` — `FindConfig()`, `FindConfigWithin()`.

## Requirements

- [x] `find_config(start_dir)` — walk up from `start_dir` to filesystem root, return path to first `.ish.yml` found
- [x] `find_config_within(start_dir, root_dir)` — same but stop at `root_dir`
- [x] Return `None` if no config found (not an error)

## Verification

```bash
cargo test
```

Unit tests with temp directories: config in current dir, config in parent, no config found.

## Summary of Changes

Implemented `find_config` and `find_config_within` in `src/config/mod.rs` using upward directory traversal for `.ish.yml` discovery. Added temp-directory unit coverage for current-directory discovery, parent-directory discovery, missing-config behavior, and the bounded root search case. Added `#[allow(dead_code)]` on the new helpers because this task lands the discovery API before a caller is wired up elsewhere in the CLI.

## Notes for Future Work

The helpers currently operate on the provided paths directly rather than canonicalizing them first, which keeps the behavior simple and matches the current tests. If future callers pass mixed relative/symlinked paths, revisit whether discovery should normalize paths before the ancestry checks.
