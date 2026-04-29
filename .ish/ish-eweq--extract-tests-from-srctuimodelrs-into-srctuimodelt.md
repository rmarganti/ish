---
# ish-eweq
title: Extract tests from src/tui/model.rs into src/tui/model/tests.rs
status: completed
type: task
priority: normal
tags:
- refactor
- tests
- maintainability
created_at: 2026-04-29T19:49:44.216100350Z
updated_at: 2026-04-29T20:07:16.087569Z
parent: ish-w1u4
---

## Goal
Extract the inline test module from `src/tui/model.rs` into dedicated test files while preserving state-model and board-row/tree shaping behavior.

## Required organization
Adopt the directory-style module layout discussed earlier:

```text
src/tui/model/
  mod.rs
  tests.rs
```

This module is a moderate future split candidate, and this layout should make later separation of enum/types, screen state, and board helpers easier if that work becomes worthwhile.

## Constraints
- Preserve the existing `crate::tui::model` API and imports.
- Keep this task focused on test extraction and organization.
- Avoid unrelated TUI model refactors.

## Acceptance criteria
- [x] `src/tui/model.rs` is replaced by `src/tui/model/mod.rs`.
- [x] The inline `#[cfg(test)]` module is moved into `src/tui/model/tests.rs`.
- [x] Existing callers continue to compile unchanged.
- [x] `mise run ci` passes.

## Implementation notes
- Converted `src/tui/model.rs` into the directory-style module layout at `src/tui/model/mod.rs`, preserving the existing `crate::tui::model` entry point while leaving room for future splits of enums, screen state, and board helpers.
- Moved the full inline `#[cfg(test)]` suite into `src/tui/model/tests.rs` and reconnected it with a minimal `#[cfg(test)] mod tests;` declaration in `mod.rs`.
- Kept the extracted tests using `super::{Model, Priority, Status}` so the bucket-ordering and board-tree-shaping coverage stays close to the production module contract without introducing broader TUI model refactors.

## Validation
- `mise exec -- cargo test tui::model -- --nocapture`
- `mise exec -- ish check`
- `mise run ci`

## Follow-up notes
- `src/tui/model/` is now prepared for future internal extraction if the TUI state layer keeps growing; follow the existing pattern of keeping production code in `mod.rs` and adding focused siblings rather than reintroducing large inline test blocks.
