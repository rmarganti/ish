---
# ish-6ye6
title: 'CLI: create command'
status: completed
type: task
priority: normal
created_at: 2026-04-17T13:35:00Z
updated_at: 2026-04-17T16:46:11Z
parent: ish-gt2k
blocked_by:
    - ish-sr0l
    - ish-qpqo
    - ish-vb4z
---

## Description\n\nImplement `ish create [title]` — create a new ishoo.\n\nReference: `beans/internal/commands/create.go`\n\n## Requirements\n\n- [x] Positional arg: title (default: "Untitled")\n- [x] Flags: `-s/--status`, `-t/--type`, `-p/--priority`, `-d/--body`, `--body-file`, `--tag` (repeatable), `--parent`, `--blocking` (repeatable), `--blocked-by` (repeatable), `--prefix`, `--json`\n- [x] `--body` and `--body-file` mutually exclusive; `--body -` reads from stdin\n- [x] Validate status, type, priority against config\n- [x] Apply config defaults for status and type if not specified\n- [x] Output: styled "Created {id} {path}" or JSON\n\n## Verification\n\n```bash\ncargo test\n```\n\nIntegration tests: create with various flag combinations, verify file on disk.

## Summary of Changes

- Added `ish create [title]` to the CLI with support for status/type/priority overrides, inline body input, `--body-file`, repeatable tags and dependency flags, parent assignment, `--prefix`, and `--json`.
- Wired the command to `Store::create`, added stdin/file body loading plus StoreError-to-CLI error classification, and returned either a styled `Created {id} {path}` message or the created ishoo in the shared JSON response envelope.
- Extended `CreateIshoo` with an ID-prefix override so `--prefix` only changes the new issue ID while parent/blocking references still resolve against the project-configured prefix.

## Notes for Future Workers

- `--prefix` is intentionally scoped to ID generation only; relation flags still normalize using the workspace config prefix, so shorthand relation IDs continue to target existing project issues.
- The create coverage lives in `src/main.rs` tests because the current CLI commands are implemented directly there rather than in separate command modules.
- Validation loop completed: `cargo fmt --all -- --check`, `cargo test`, and `cargo clippy -- -D warnings`.
