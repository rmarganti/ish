---
# ish-3o4l
title: 'CLI: check command'
status: completed
type: task
priority: normal
created_at: 2026-04-17T13:35:00Z
updated_at: 2026-04-17T16:34:21Z
parent: ish-gt2k
blocked_by:
    - ish-h9yk
    - ish-qpqo
    - ish-vb4z
---

## Description\n\nImplement `ish check` — validate configuration and link integrity.\n\nReference: `beans/internal/commands/check.go`\n\n## Requirements\n\n- [x] Config checks: validate default_status, default_type, colors\n- [x] Link checks: broken links, self-references, cycles (via `check_all_links()`)\n- [x] `--fix` — auto-fix broken links and self-references (not cycles)\n- [x] `--json` — JSON output with structured result\n- [x] Exit code 1 if issues found\n- [x] Human output: ✓/✗ for each check, summary at end\n\n## Verification\n\n```bash\ncargo test\n```\n\nIntegration tests: create ishoos with broken links, run check, verify detection. Run check --fix, verify fixes applied.\n\n## Summary of Changes\n\n- Added a new `ish check` CLI command with `--fix` support, structured JSON output, and non-zero exit status when validation finds issues.\n- Reused the store link-checking and link-fixing primitives to report broken links, self-references, and cycles, and added config validation for default type/status and terminal color mappings.\n- Added command-level regression tests covering clean runs, failing runs, JSON output, and `--fix` behavior.\n\n## Notes for Future Workers\n\n- `ish check --fix` intentionally fixes broken links and self-references by rewriting issue files, but it does not attempt to resolve cycles; cycles remain report-only.\n- The command exits with code 1 when issues were detected before fixing, even if `--fix` clears the remaining broken/self links, so callers can still treat the run as a validation failure.\n- JSON output nests both `initial` and `final` link check results under `data.checks.links`, which should be preserved if future work adds more check categories.\n\n## Verification Notes\n\n- Ran `cargo fmt --all -- --check`\n- Ran `cargo test`\n- Ran `cargo clippy -- -D warnings`
