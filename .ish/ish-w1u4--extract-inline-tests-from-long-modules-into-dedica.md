---
# ish-w1u4
title: Extract inline tests from long modules into dedicated test files
status: completed
type: epic
priority: normal
tags:
- refactor
- tests
- maintainability
created_at: 2026-04-29T19:49:07.712571528Z
updated_at: 2026-04-29T20:10:30.162604Z
---

## Summary
Extract large inline `#[cfg(test)]` modules into dedicated test files for the longest and highest-context modules in the repo.

## Context
We reviewed the repo's longer files and concluded that several modules are meaningfully harder to read because implementation and extensive test suites live in the same file. The most valuable first step is test extraction, even before deeper implementation splitting.

The agreed organization for modules that are also likely future implementation-split candidates is:

- prefer directory-style modules now
- move implementation to `mod.rs`
- move tests to `tests.rs`
- preserve the current public API and behavior
- keep test coverage intact
- avoid mixing this work with unrelated refactors

## Scope
This epic tracks one child ish per target module:

- `src/core/store.rs`
- `src/tui/update.rs`
- `src/model/ish.rs`
- `src/output/mod.rs`
- `src/roadmap.rs`
- `src/tui/model.rs`

## Acceptance criteria
- [x] Each target module has its inline tests extracted into a dedicated test file.
- [x] File/module organization matches the agreed directory-style layout where practical.
- [x] Existing imports and module paths continue to compile without consumer-facing behavior changes.
- [x] Validation passes with `mise run ci` after each child ish is completed.

## Implementation notes
- Completed the full child set across `src/core/store/`, `src/tui/update/`, `src/model/ish/`, `src/output/`, `src/roadmap/`, and `src/tui/model/`, consistently moving inline `#[cfg(test)]` suites into sibling `tests.rs` files.
- Standardized on directory-style modules where the target file was a strong future split candidate, which keeps the current public module paths stable while leaving room for later internal extraction.
- The final roadmap extraction (`ish-xnnj`) finished the sweep and confirmed the pattern still compiles cleanly for command-layer callers that depend on `crate::roadmap`.

## Validation
- Child-task validations recorded in each ish
- `mise exec -- ish check`
- `mise run ci`

## Follow-up notes
- Future internal splits should follow the established pattern: keep the top-level module path stable, move focused implementation into sibling files only when it materially improves readability, and leave tests in dedicated `tests.rs` modules rather than reintroducing large inline test blocks.
