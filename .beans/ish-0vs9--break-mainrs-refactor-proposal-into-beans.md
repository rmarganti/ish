---
# ish-0vs9
title: Break main.rs refactor proposal into beans
status: completed
type: task
priority: normal
created_at: 2026-04-17T19:00:41Z
updated_at: 2026-04-17T19:02:14Z
---

Create a structured set of beans for the proposed main.rs modularization work, including context, dependencies, and verification steps.

## Summary of Changes

Created a new epic and a staged set of child/follow-up beans for the `main.rs` modularization proposal:
- `ish-hqty` — epic for the overall refactor
- `ish-t1q0` — extract clap types into `src/cli/args.rs`
- `ish-nfx6` — introduce `src/app/` context/error/dispatch modules
- `ish-oewf` — move command handlers into `src/commands/`
- `ish-ywbj` — split `list` and `check` into focused submodules
- `ish-adhz` — slim `main.rs` and redistribute tests
- `ish-6bsa` — draft follow-up for second-stage splits (`roadmap`, `output`, `store`)

Each bean includes:
- current codebase context
- scoped goals and success criteria
- explicit verification steps using the project feedback loop
- dependency links so the work can proceed in a sensible order
