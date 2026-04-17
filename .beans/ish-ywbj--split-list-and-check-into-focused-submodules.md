---
# ish-ywbj
title: Split list and check into focused submodules
status: todo
type: task
priority: normal
created_at: 2026-04-17T19:01:44Z
updated_at: 2026-04-17T19:02:06Z
parent: ish-hqty
blocked_by:
    - ish-oewf
---

## Goal

Break the heaviest command implementations into smaller units so command files do not become new god-files after the extraction from `src/main.rs`.

## Context

Two command areas already show strong internal seams:
- `list`: argument validation, filtering/matching, sorting, JSON shaping, and tree rendering preparation
- `check`: config validation, link issue summarization, human rendering, and JSON rendering

If these stay as single files, the refactor will only relocate complexity instead of reducing it.

## Scope

- Split `list` into `src/commands/list/mod.rs`, `filters.rs`, and `render.rs`.
- Split `check` into `src/commands/check/mod.rs`, `config.rs`, and `render.rs`.
- Keep helpers close to the feature that owns them.
- Avoid moving command-specific rendering into generic `src/output/` unless it is genuinely reusable.

## Success Criteria

- [ ] `list` filtering/matching logic is isolated from its output shaping.
- [ ] `check` validation logic is isolated from rendering logic.
- [ ] Resulting files are materially smaller and easier to navigate than the original chunks in `src/main.rs`.

## Verification

- [ ] `cargo test`
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy -- -D warnings`
- [ ] Existing list/check tests still pass, with new tests added near the extracted modules if gaps appear.
