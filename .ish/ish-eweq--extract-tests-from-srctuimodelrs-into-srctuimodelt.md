---
# ish-eweq
title: Extract tests from src/tui/model.rs into src/tui/model/tests.rs
status: todo
type: task
priority: normal
tags:
- refactor
- tests
- maintainability
created_at: 2026-04-29T19:49:44.216100350Z
updated_at: 2026-04-29T19:49:44.216100350Z
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
- [ ] `src/tui/model.rs` is replaced by `src/tui/model/mod.rs`.
- [ ] The inline `#[cfg(test)]` module is moved into `src/tui/model/tests.rs`.
- [ ] Existing callers continue to compile unchanged.
- [ ] `mise run ci` passes.
