---
# ish-xnnj
title: Extract tests from src/roadmap.rs into src/roadmap/tests.rs
status: todo
type: task
priority: normal
tags:
- refactor
- tests
- maintainability
created_at: 2026-04-29T19:49:44.199274474Z
updated_at: 2026-04-29T19:49:44.199274474Z
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
- [ ] `src/roadmap.rs` is replaced by `src/roadmap/mod.rs`.
- [ ] The inline `#[cfg(test)]` module is moved into `src/roadmap/tests.rs`.
- [ ] Existing callers continue to compile unchanged.
- [ ] `mise run ci` passes.
