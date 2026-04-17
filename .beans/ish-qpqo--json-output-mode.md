---
# ish-qpqo
title: JSON output mode
status: completed
type: task
priority: normal
created_at: 2026-04-17T13:33:58Z
updated_at: 2026-04-17T16:20:18Z
parent: ish-1of2
blocked_by:
    - ish-gqts
    - ish-0iv6
---

## Description\n\nImplement structured JSON output for all CLI commands.\n\nReference: `beans/internal/output/` — `Success()`, `Error()`, `SuccessMultiple()`.\n\n## Requirements\n\n- [x] `Response` struct: `success: bool`, `message: Option<String>`, `data: Option<T>` (single ishoo), `ishoos: Option<Vec<Ishoo>>` (list), `count: Option<usize>`\n- [x] Error codes: `not_found`, `validation`, `conflict`, `file_error`\n- [x] `output_success(ishoo)`, `output_success_multiple(ishoos)`, `output_error(code, message)`\n- [x] All output goes to stdout as pretty-printed JSON\n- [x] Every CLI command supports `--json` flag to switch to this mode\n\n## Verification\n\n```bash\ncargo test\n```\n\nUnit tests: verify JSON structure for success and error cases.

## Summary of Changes

- Added `src/output/mod.rs` with a shared pretty-printed JSON response envelope, response metadata, and standardized error codes.
- Promoted `--json` to a global CLI flag so every implemented command (`prime`, `roadmap`, `version`) can switch to structured output.
- Wrapped roadmap JSON under the shared response envelope and added regression tests for JSON success and error cases.

## Notes for Future Workers

- `roadmap` already had a nested JSON representation; `main` now parses that command-specific payload and re-wraps it in the shared response envelope.
- `output_success_multiple()` is implemented for upcoming list/show-style commands but is not wired into a command yet, so it is marked `#[allow(dead_code)]` until those commands land.
- `prime` still exits silently when no `.ish.yml` exists; JSON mode preserves that behavior for now because there is no existing project context to report from.
