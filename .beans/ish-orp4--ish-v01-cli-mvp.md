---
# ish-orp4
title: ish v0.1 — CLI MVP
status: completed
type: milestone
priority: normal
created_at: 2026-04-17T13:30:30Z
updated_at: 2026-04-17T18:56:29Z
blocked_by:
    - ish-m4jr
    - ish-4qx4
    - ish-ksgz
    - ish-6cqj
    - ish-idzc
    - ish-1of2
    - ish-gt2k
---

First usable release of ish: a terminal-based issue tracker written in Rust. Reimplements the core CLI features of Beans (hmans/beans). Issues ("ishoos") are markdown files with YAML frontmatter stored in a `.ish/` directory.\n\nScope: CLI only (no GraphQL, no web UI, no worktree management). TUI to follow after CLI is complete.\n\nKey decisions:\n- App name: `ish`, issue equivalent: `ishoo`\n- Storage path: `.ish/` (not `.beans/`)\n- Config file: `.ish.yml`\n- No GraphQL layer — CLI calls core engine directly\n- Simple substring search (not Bleve/Tantivy)\n- No need to maintain Beans file compatibility

## Summary of Changes

Completed the `ish v0.1` CLI MVP milestone. The project now includes the planned storage layer, config loading, relationship validation, output/rendering support, and the initial command set (`init`, `create`, `list`, `show`, `update`, `delete`, `archive`, `check`, `roadmap`, `prime`, `version`).

## Notes for Future Workers

- The MVP scope is fully CLI-only; any next phase work should build on the existing command and output abstractions rather than introducing a parallel interface.
- The current implementation favors comprehensive unit/integration-style tests in `src/main.rs` plus module-local tests; preserve that pattern when extending behavior.
- Final milestone validation passed with `cargo test`, `cargo fmt --all -- --check`, and `cargo clippy -- -D warnings`.
