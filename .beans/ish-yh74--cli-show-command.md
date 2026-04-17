---
# ish-yh74
title: 'CLI: show command'
status: completed
type: task
priority: normal
created_at: 2026-04-17T13:35:00Z
updated_at: 2026-04-17T18:54:24Z
parent: ish-gt2k
blocked_by:
    - ish-ffou
    - ish-qpqo
    - ish-vb4z
    - ish-bpxr
    - ish-2iuf
---

## Description\n\nImplement `ish show <id> [id...]` — display full ishoo details.\n\nReference: `beans/internal/commands/show.go`\n\n## Requirements\n\n- [x] Accept one or more IDs\n- [x] Flags: `--json`, `--raw` (raw markdown), `--body-only`, `--etag-only` (mutually exclusive)\n- [x] Default styled output: header (ID, status badge, type badge, priority badge, tags, timestamps), separator, relationships (parent, blocking), separator, rendered markdown body\n- [x] Multiple ishoos separated by `═` line\n- [x] Short ID support (auto-prepend prefix)\n\n## Verification\n\n```bash\ncargo test\n```\n\nIntegration tests: show with each output mode, verify format.

## Summary of Changes

- Added the `ish show <id> [id...]` CLI command with short-ID lookup, detailed human-readable output, and support for multiple ishoos in one invocation.
- Implemented `--json`, `--raw`, `--body-only`, and `--etag-only` output modes, using the existing structured response helpers for JSON output.
- Reused the shared badge and markdown rendering helpers so show output includes styled headers, relationship details, rendered body content, inherited status context, and multi-ishoo separators.
- Added parser and command tests covering the new output modes plus human-readable formatting, and updated the prime guide now that `ish show` is supported.
- Validation passed: `cargo test`, `cargo fmt --all -- --check`, and `cargo clippy -- -D warnings`.

## Notes for Future Workers

- Human `ish show` output is assembled in `src/main.rs` via `render_show_human()` and currently includes extra relationship context (`blocked_by`, incoming links, inherited status, active blockers`) beyond the minimum bean requirements.
- JSON show output currently uses the multi-item response shape (`ishoos` + `count`) even for a single requested ID, which keeps the multi-ID command behavior consistent.
- The prime guide in `src/cli/mod.rs` now documents `ish show`; if the command surface changes again, keep that list in sync with the actual CLI.
