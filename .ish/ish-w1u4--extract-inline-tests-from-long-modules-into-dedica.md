---
# ish-w1u4
title: Extract inline tests from long modules into dedicated test files
status: todo
type: epic
priority: normal
tags:
- refactor
- tests
- maintainability
created_at: 2026-04-29T19:49:07.712571528Z
updated_at: 2026-04-29T19:49:07.712571528Z
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
- [ ] Each target module has its inline tests extracted into a dedicated test file.
- [ ] File/module organization matches the agreed directory-style layout where practical.
- [ ] Existing imports and module paths continue to compile without consumer-facing behavior changes.
- [ ] Validation passes with `mise run ci` after each child ish is completed.
