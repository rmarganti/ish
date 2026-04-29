---
# ish-0zek
title: Extract tests from src/model/ish.rs into src/model/ish/tests.rs
status: todo
type: task
priority: normal
tags:
- refactor
- tests
- maintainability
created_at: 2026-04-29T19:49:44.156752159Z
updated_at: 2026-04-29T19:49:44.156752159Z
parent: ish-w1u4
---

## Goal
Extract the inline test module from `src/model/ish.rs` into dedicated test files while preserving parse/render, tag, body, ID, and ordering behavior.

## Required organization
Adopt the directory-style module layout discussed earlier:

```text
src/model/ish/
  mod.rs
  tests.rs
```

This module is also a good future split candidate, and the new layout should make later extraction into focused files like `frontmatter.rs`, `tags.rs`, `body.rs`, `identity.rs`, and `order.rs` straightforward.

## Constraints
- Preserve the existing `crate::model::ish` public API.
- Keep this task focused on test extraction and file organization.
- Do not mix in broader model refactors.

## Acceptance criteria
- [ ] `src/model/ish.rs` is replaced by `src/model/ish/mod.rs`.
- [ ] The inline `#[cfg(test)]` module is moved into `src/model/ish/tests.rs`.
- [ ] Existing callers continue to compile unchanged.
- [ ] Test intent and coverage remain intact.
- [ ] `mise run ci` passes.
