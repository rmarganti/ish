---
# ish-adhz
title: Slim main.rs and redistribute tests after extraction
status: completed
type: task
priority: high
created_at: 2026-04-17T19:01:52Z
updated_at: 2026-04-17T19:37:24Z
parent: ish-hqty
blocked_by:
    - ish-ywbj
---

## Goal

Finish the refactor by reducing `src/main.rs` to a minimal entrypoint and relocating oversized tests closer to the modules they exercise.

## Context

Even after moving code, the refactor is not complete unless the entrypoint becomes obviously small and maintainable. `src/main.rs` currently also hosts a very large test module, which will continue to obscure intent if it remains centralized.

Desired end state:
- `src/main.rs` only parses args, calls the app runner, prints output, and returns an `ExitCode`.
- Command tests move near their owning command modules where practical.
- Shared app/error/context tests move near `src/app/`.

## Scope

- Remove lingering implementation details from `src/main.rs`.
- Move tests out of the giant `main.rs` test module into feature-local modules where feasible.
- Leave only the smallest bootstrap logic in `src/main.rs`.
- Confirm the final hierarchy matches the refactor intent.

## Success Criteria

- [x] `src/main.rs` is small enough to read top-to-bottom quickly.
- [x] Most tests live near the code they exercise.
- [x] No new god-file has replaced `src/main.rs`.

## Verification

- [x] `cargo test`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy -- -D warnings`
- [x] Manual spot-check: opening `src/main.rs` shows bootstrap only, not feature logic.

## Summary of Changes

- Reduced `src/main.rs` to module declarations plus the CLI bootstrap path only.
- Moved the oversized `main.rs` test coverage into feature-local `#[cfg(test)]` modules across `src/app/`, `src/cli/`, and the relevant `src/commands/*` files.
- Added `src/test_support.rs` for shared temp-dir / working-directory / fixture helpers so test setup stays localized without rebuilding another god-file.
- Ran the full feedback loop successfully (`cargo test`, `cargo fmt --all -- --check`, `cargo clippy -- -D warnings`).
- Note for future workers: run the feedback loop sequentially; a parallel `cargo test` + `cargo clippy` invocation briefly exposed a temp-dir test failure even though the standard sequential loop passes.
