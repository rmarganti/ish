---
# ish-nmma
title: Fix flaky output test caused by global color override
status: completed
type: task
priority: low
created_at: 2026-04-19T03:39:48.709931Z
updated_at: 2026-04-19T12:43:13.090463Z
---

- Investigate global `colored::control::set_override(...)` test interference in `src/output/mod.rs`.
- Make output tests deterministic under parallel `cargo test`.
- Reproduce and fix `render_tree_uses_connectors_implicit_status_tags_and_truncation` flake.



## Implementation notes
- Added a `with_color_override(...)` test helper in `src/output/mod.rs` that serializes access to `colored::control::set_override(...)` with a `Mutex` and restores the previous setting via `Drop`.
- Wrapped the output tests that toggle the global color override so parallel `cargo test` runs cannot leak ANSI/no-ANSI state across tests.
- Reproduced the original flake by looping `cargo test output::tests:: -- --test-threads=8`; the failure was `render_tree_uses_connectors_implicit_status_tags_and_truncation` intermittently missing the plain `└── ish-child` connector when another test enabled color output concurrently.

## Validation
- `python3` stress loop: 30x `mise exec -- cargo test output::tests:: -- --test-threads=8`
- `mise run ci`
