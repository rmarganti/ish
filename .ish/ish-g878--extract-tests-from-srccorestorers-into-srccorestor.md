---
# ish-g878
title: Extract tests from src/core/store.rs into src/core/store/tests.rs
status: completed
type: task
priority: normal
tags:
- refactor
- tests
- maintainability
created_at: 2026-04-29T19:49:44.117859900Z
updated_at: 2026-04-29T19:49:44.117859900Z
parent: ish-w1u4
---

## Goal
Extract the inline test module from `src/core/store.rs` into dedicated test files while preserving behavior and setting the module up for future submodule extraction.

## Required organization
Adopt the directory-style module layout discussed earlier:

```text
src/core/store/
  mod.rs
  tests.rs
```

This module is the strongest future split candidate in the repo, so this task should establish the layout that can later grow into `persistence.rs`, `mutations.rs`, `validation.rs`, and `links.rs` without another naming churn.

## Constraints
- Preserve the existing `crate::core::store` public API.
- Keep this task focused on test extraction and module/file organization only.
- Do not change runtime behavior unless required to keep tests compiling after the move.
- Keep implementation in `mod.rs` for now; deeper implementation splitting is out of scope.

## Acceptance criteria
- [x] `src/core/store.rs` is replaced by `src/core/store/mod.rs`.
- [x] The inline `#[cfg(test)]` module is moved into `src/core/store/tests.rs`.
- [x] `mod tests;` wiring is added cleanly.
- [x] Tests continue to exercise the same behaviors.
- [x] `mise run ci` passes.

## Implementation notes
- Converted `src/core/store.rs` into the directory-style module layout at `src/core/store/mod.rs` so the existing `crate::core::store` API stays stable while making future submodule splits straightforward.
- Moved the full inline store test suite into `src/core/store/tests.rs` and kept it as a sibling test module via `#[cfg(test)] mod tests;`.
- Kept the extracted tests using `super::...` imports, which minimizes churn if later work splits store internals into smaller files under `src/core/store/`.

## Validation
- `mise exec -- cargo test core::store -- --nocapture`
- `mise run ci`
- `mise exec -- ish check`

## Follow-up notes
- `src/core/store/mod.rs` is now ready for targeted internal extraction (for example persistence/link validation helpers) without another public module rename or a second test move.
