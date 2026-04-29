---
# ish-w6lv
title: Extract tests from src/output/mod.rs into src/output/tests.rs
status: todo
type: task
priority: normal
tags:
- refactor
- tests
- maintainability
created_at: 2026-04-29T19:49:44.177279676Z
updated_at: 2026-04-29T19:49:44.177279676Z
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
- [ ] The inline `#[cfg(test)]` module is moved into `src/output/tests.rs`.
- [ ] `mod tests;` wiring is added cleanly.
- [ ] Production code behavior stays unchanged.
- [ ] `mise run ci` passes.
