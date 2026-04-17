---
# ish-ywbj
title: Split list and check into focused submodules
status: completed
type: task
priority: normal
created_at: 2026-04-17T19:01:44Z
updated_at: 2026-04-17T19:27:05Z
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

- [x] `list` filtering/matching logic is isolated from its output shaping.
- [x] `check` validation logic is isolated from rendering logic.
- [x] Resulting files are materially smaller and easier to navigate than the original chunks in `src/main.rs`.

## Verification

- [x] `cargo test`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy -- -D warnings`
- [x] Existing list/check tests still pass, with new tests added near the extracted modules if gaps appear.

## Summary of Changes

- Split `src/commands/list.rs` into `src/commands/list/mod.rs`, `filters.rs`, and `render.rs`, keeping argument validation and filtering separate from JSON/tree rendering.
- Split `src/commands/check.rs` into `src/commands/check/mod.rs`, `config.rs`, and `render.rs`, separating config/link validation from human and JSON output formatting.
- Kept the public command entrypoints stable so the app dispatcher and existing tests in `src/main.rs` continued to work unchanged after the refactor.

## Notes for Future Workers

- `list` tree rendering still depends on `crate::output::build_tree` / `render_tree`; if more list-specific presentation logic accumulates, continue growing `src/commands/list/render.rs` before pushing feature-specific code into generic output helpers.
- `check` now has a clear seam between validation (`config.rs`) and presentation (`render.rs`), which should make it easier to add extra checks or output fields without re-entangling CLI formatting logic.
