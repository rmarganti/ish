---
# ish-senq
title: Extract tests from src/tui/update.rs into src/tui/update/tests.rs
status: todo
type: task
priority: normal
tags:
- refactor
- tests
- maintainability
created_at: 2026-04-29T19:49:44.138391171Z
updated_at: 2026-04-29T19:49:44.138391171Z
parent: ish-w1u4
---

## Goal
Extract the inline test module from `src/tui/update.rs` into dedicated test files while preserving the current reducer/update behavior.

## Required organization
Adopt the directory-style module layout discussed earlier:

```text
src/tui/update/
  mod.rs
  tests.rs
```

This module is a strong future split candidate by screen/global behavior, so this layout should leave room for later files like `global.rs`, `board.rs`, `detail.rs`, `picker.rs`, and `create_form.rs`.

## Constraints
- Preserve the existing `crate::tui::update` entry points and behavior.
- Keep this task limited to test extraction and module/file organization.
- Avoid functional reducer changes except those strictly needed to support the move.

## Acceptance criteria
- [ ] `src/tui/update.rs` is replaced by `src/tui/update/mod.rs`.
- [ ] The inline `#[cfg(test)]` module is moved into `src/tui/update/tests.rs`.
- [ ] Module wiring remains clean and idiomatic.
- [ ] Existing test coverage remains intact.
- [ ] `mise run ci` passes.
