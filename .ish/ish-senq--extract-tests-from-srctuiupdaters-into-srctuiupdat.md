---
# ish-senq
title: Extract tests from src/tui/update.rs into src/tui/update/tests.rs
status: completed
type: task
priority: normal
tags:
- refactor
- tests
- maintainability
created_at: 2026-04-29T19:49:44.138391171Z
updated_at: 2026-04-29T19:57:29.119684Z
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
- [x] `src/tui/update.rs` is replaced by `src/tui/update/mod.rs`.
- [x] The inline `#[cfg(test)]` module is moved into `src/tui/update/tests.rs`.
- [x] Module wiring remains clean and idiomatic.
- [x] Existing test coverage remains intact.
- [x] `mise run ci` passes.

## Implementation notes
- Converted `src/tui/update.rs` into the directory-style module layout at `src/tui/update/mod.rs`, preserving the existing `crate::tui::update` entry point while making later behavior-specific extractions possible without another module rename.
- Moved the full inline reducer test suite into `src/tui/update/tests.rs` and kept it connected with a small `#[cfg(test)] mod tests;` declaration in `mod.rs`.
- Left the test helper functions colocated in `tests.rs`, so future splits into `global.rs`, `board.rs`, `detail.rs`, `picker.rs`, and `create_form.rs` can move implementation without having to unwind a mixed production/test file again.

## Validation
- `mise exec -- cargo test tui::update -- --nocapture`
- `mise exec -- ish check`
- `mise run ci`

## Follow-up notes
- `src/tui/update/mod.rs` is now ready for deeper internal extraction by reducer concern if this module keeps growing; the current split keeps behavior unchanged while removing ~600 lines of test code from the production file.
