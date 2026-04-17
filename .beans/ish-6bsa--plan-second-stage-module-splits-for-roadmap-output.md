---
# ish-6bsa
title: Plan second-stage module splits for roadmap, output, and store
status: draft
type: task
priority: normal
created_at: 2026-04-17T19:02:01Z
updated_at: 2026-04-17T19:02:06Z
blocked_by:
    - ish-adhz
---

## Goal

Capture the likely follow-up modularization work that should be evaluated after the primary `main.rs` / command extraction lands.

## Context

Current codebase observations suggest future candidates for additional splitting:
- `src/roadmap.rs` could become `src/roadmap/{mod,build,render}.rs`
- `src/output/mod.rs` could separate generic JSON/style/markdown/tree concerns
- `src/core/store.rs` is already large and may benefit from `load`, `mutate`, `links`, and `archive` submodules

This work should not block the first refactor. The team should first land the command/app extraction and then reevaluate what still feels too large.

## Scope

- Reassess file sizes and cohesion after the command extraction lands.
- Decide whether `roadmap`, `output`, and/or `store` need immediate follow-up splits.
- If needed, create concrete implementation beans for the selected follow-up work.

## Success Criteria

- [ ] Follow-up modularization is documented but does not prematurely complicate the first refactor.
- [ ] Any second-stage split is driven by post-refactor pain points, not speculation alone.

## Verification

- [ ] Re-read the post-refactor file tree and identify remaining oversized or mixed-responsibility modules.
- [ ] If follow-up work is warranted, create new beans with scoped success criteria and verification steps.
