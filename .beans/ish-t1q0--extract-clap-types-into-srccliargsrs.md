---
# ish-t1q0
title: Extract clap types into src/cli/args.rs
status: completed
type: task
priority: high
created_at: 2026-04-17T19:01:17Z
updated_at: 2026-04-17T19:05:39Z
parent: ish-hqty
---

## Goal

Move CLI declarations out of `src/main.rs` into `src/cli/args.rs` so argument shapes, clap attributes, and CLI-facing enums are isolated from application logic.

## Context

Relevant items currently in `src/main.rs`:
- `Cli`
- `Commands`
- `CreateArgs`, `ListArgs`, `UpdateArgs`, `ShowArgs`, `DeleteArgs`, `CheckArgs`, `RoadmapArgs`
- `ListSortArg`

This is the lowest-risk first extraction because it is mostly declarative and reduces noise before moving behavior.

## Scope

- Create `src/cli/args.rs`.
- Move all clap structs/enums there.
- Update imports/re-exports so the rest of the app compiles cleanly.
- Keep existing clap flags, aliases, conflicts, and help text unchanged.

## Success Criteria

- [x] `src/main.rs` no longer defines clap structs/enums.
- [x] `src/cli/mod.rs` cleanly exposes the args module.
- [x] CLI parsing behavior remains unchanged.

## Verification

- [x] `cargo test`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy -- -D warnings`
- [x] Spot-check `cargo run -- --help` output if needed to confirm flags/aliases still render correctly.

## Summary of Changes

- Extracted all clap parser structs/enums from `src/main.rs` into the new `src/cli/args.rs` module.
- Re-exported CLI arg types from `src/cli/mod.rs` so the rest of the application can keep importing them from `crate::cli`.
- Verified the refactor with `cargo test`, `cargo fmt --all -- --check`, `cargo clippy -- -D warnings`, and a `cargo run -- --help` spot-check to confirm the CLI surface stayed unchanged.
