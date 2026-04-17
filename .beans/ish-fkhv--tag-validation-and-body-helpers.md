---
# ish-fkhv
title: Tag validation and body helpers
status: completed
type: task
priority: normal
created_at: 2026-04-17T13:31:37Z
updated_at: 2026-04-17T15:30:49Z
parent: ish-4qx4
blocked_by:
    - ish-0iv6
---

## Description

Implement tag validation/normalization and body content manipulation helpers.

Reference: `beans/pkg/bean/bean.go` (tags) and `beans/pkg/bean/content.go` (body helpers).

## Requirements

Tags:
- [x] `validate_tag(tag)` — must match `^[a-z][a-z0-9]*(-[a-z0-9]+)*$`
- [x] `normalize_tag(tag)` — lowercase + trim
- [x] `has_tag()`, `add_tag()`, `remove_tag()` methods on Ishoo

Body helpers:
- [x] `replace_once(text, old, new)` — error if old is empty, not found, or found multiple times
- [x] `unescape_body(s)` — interpret `\n`, `\t`, `\\` escape sequences
- [x] `append_with_separator(text, addition)` — join with blank line separator

## Verification

```bash
cargo fmt --all -- --check
cargo test
cargo clippy -- -D warnings
```

Unit tests cover valid, invalid, and edge-case behavior for tag normalization/validation and body helper functions.

## Summary of Changes

Added tag normalization and validation helpers plus `Ishoo::{has_tag, add_tag, remove_tag}` in `src/model/ishoo.rs`. Added body manipulation helpers for single-match replacement, escaped newline/tab/backslash expansion, and blank-line appends, with unit tests covering valid, invalid, and edge-case behavior.

## Verification Notes

Validated with `cargo fmt --all -- --check`, `cargo test`, and `cargo clippy -- -D warnings` after formatting the file.
