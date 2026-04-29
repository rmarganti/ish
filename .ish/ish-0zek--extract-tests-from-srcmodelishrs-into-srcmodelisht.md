---
# ish-0zek
title: Extract tests from src/model/ish.rs into src/model/ish/tests.rs
status: completed
type: task
priority: normal
tags:
- refactor
- tests
- maintainability
created_at: 2026-04-29T19:49:44.156752159Z
updated_at: 2026-04-29T20:00:10.903842Z
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
- [x] `src/model/ish.rs` is replaced by `src/model/ish/mod.rs`.
- [x] The inline `#[cfg(test)]` module is moved into `src/model/ish/tests.rs`.
- [x] Existing callers continue to compile unchanged.
- [x] Test intent and coverage remain intact.
- [x] `mise run ci` passes.

## Implementation notes
- Converted `src/model/ish.rs` into the directory-style module layout at `src/model/ish/mod.rs`, preserving the existing `crate::model::ish` module path while making later focused splits easier.
- Moved the full inline `#[cfg(test)]` suite into `src/model/ish/tests.rs` and reconnected it with a small `#[cfg(test)] mod tests;` declaration in `mod.rs`.
- Kept the extracted tests using `super::*` so future internal extraction of frontmatter/tag/body/ordering helpers can happen without rewriting the test module imports.

## Validation
- `mise exec -- cargo test model::ish -- --nocapture`
- `mise exec -- ish check`
- `mise run ci`

## Follow-up notes
- `src/model/ish/mod.rs` is now ready for future internal extraction into smaller helpers like frontmatter, tags, body, identity, or order logic without another top-level module rename.
