---
# ish-4b45
title: 'CLI: prime command'
status: completed
type: task
priority: normal
created_at: 2026-04-17T13:35:00Z
updated_at: 2026-04-17T18:27:39Z
parent: ish-gt2k
blocked_by:
    - ish-614c
---

## Description\n\nImplement `ish prime` — output agent prompt instructions.\n\nReference: `beans/internal/commands/prime.go` and `prompt.tmpl`.\n\n## Requirements\n\n- [x] Template-based output that teaches AI agents how to use the `ish` CLI\n- [x] Include: CLI command reference, types, statuses, priorities, body modification guide, concurrency control, relationship model\n- [x] Use ish/ishoo terminology throughout (not beans)\n- [x] Silently exit if no `.ish.yml` found (don't error)\n- [x] Populate template with hardcoded types, statuses, priorities from config\n\n## Verification\n\n- `cargo test`\n- `cargo fmt --all -- --check`\n- `cargo clippy -- -D warnings`\n- Manual: `cargo run -- prime`\n\n## Summary of Changes\n\n- Removed the unsupported `ish show` entry from the generated prime guide so the command reference matches the actual CLI surface.\n- Clarified the machine-readable output guidance and documented the concrete `ish update --if-match <etag>` concurrency flow using current CLI support.\n- Added test coverage to keep the rendered guide aligned with config-driven output and supported commands.
