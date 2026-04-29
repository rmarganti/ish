---
# ish-w6lv
title: Extract tests from src/output/mod.rs into src/output/tests.rs
status: completed
type: task
priority: normal
tags:
- refactor
- tests
- maintainability
created_at: 2026-04-29T19:49:44.177279676Z
updated_at: 2026-04-29T20:04:46.583893Z
parent: ish-w1u4
---

## Goal
Extract the inline test module from `src/output/mod.rs` into a dedicated test file while preserving JSON output, tree rendering, style helpers, and markdown rendering behavior.

## Required organization
Keep the existing directory-style module and add a dedicated test file:

```text
src/output/
  mod.rs
  tests.rs
```

This should improve readability now and leave the module ready for future extraction into `json.rs`, `tree.rs`, `style.rs`, and `markdown.rs` if desired later.

## Constraints
- Preserve the existing `crate::output` public API.
- Keep this work limited to test extraction and organization.
- Avoid unrelated output/rendering refactors.

## Acceptance criteria
- [x] The inline `#[cfg(test)]` module is moved into `src/output/tests.rs`.
- [x] `mod tests;` wiring is added cleanly.
- [x] Production code behavior stays unchanged.
- [x] `mise run ci` passes.

## Implementation notes
- Extracted the full inline `#[cfg(test)]` suite from `src/output/mod.rs` into the new sibling file `src/output/tests.rs`, keeping the existing `crate::output` module path unchanged.
- Replaced the old inline test block with a minimal `#[cfg(test)] mod tests;` declaration so production code in `src/output/mod.rs` stays focused on output/rendering logic.
- Preserved the existing output test helpers and coverage, including the serialized color-override guard that keeps the markdown/tree rendering tests deterministic under parallel test execution.

## Validation
- `mise exec -- cargo test output:: -- --nocapture`
- `mise run ci`
- `mise exec -- ish check`

## Follow-up notes
- `src/output/` is now organized for future internal extraction into focused files like `json.rs`, `tree.rs`, `style.rs`, or `markdown.rs` without having to unwind a mixed production/test module first.
