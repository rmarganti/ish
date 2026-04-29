---
# ish-xnnj
title: Extract tests from src/roadmap.rs into src/roadmap/tests.rs
status: completed
type: task
priority: normal
tags:
- refactor
- tests
- maintainability
created_at: 2026-04-29T19:49:44.199274474Z
updated_at: 2026-04-29T20:10:30.147091Z
parent: ish-w1u4
---

## Goal
Extract the inline test module from `src/roadmap.rs` into dedicated test files while preserving roadmap grouping, filtering, markdown output, and JSON output behavior.

## Required organization
Adopt the directory-style module layout discussed earlier:

```text
src/roadmap/
  mod.rs
  tests.rs
```

This layout should leave room for later separation into build/render/json-focused submodules without another top-level rename.

## Constraints
- Preserve the existing `crate::roadmap` public API.
- Keep this task limited to test extraction and file organization.
- Avoid changing roadmap semantics unless strictly necessary for the move.

## Acceptance criteria
- [x] `src/roadmap.rs` is replaced by `src/roadmap/mod.rs`.
- [x] The inline `#[cfg(test)]` module is moved into `src/roadmap/tests.rs`.
- [x] Existing callers continue to compile unchanged.
- [x] `mise run ci` passes.

## Implementation notes
- Converted `src/roadmap.rs` into the directory-style module layout at `src/roadmap/mod.rs`, preserving the existing `crate::roadmap` module path used by `src/main.rs` and the roadmap command.
- Moved the inline roadmap test suite into `src/roadmap/tests.rs` and reconnected it with a minimal `#[cfg(test)] mod tests;` declaration in `mod.rs`.
- Kept the extracted tests using `super::{...}` imports so future roadmap-specific splits (for example build/render/json helpers) can happen without another test relocation.

## Validation
- `mise exec -- cargo test roadmap -- --nocapture`
- `mise exec -- ish check`
- `mise run ci`

## Follow-up notes
- `src/roadmap/` is now organized for future internal extraction into focused helpers while keeping the current public API stable.
