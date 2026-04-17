---
# ish-qpqo
title: JSON output mode
status: todo
type: task
created_at: 2026-04-17T13:33:58Z
updated_at: 2026-04-17T13:33:58Z
parent: ish-1of2
blocked_by:
    - ish-gqts
    - ish-0iv6
---

## Description\n\nImplement structured JSON output for all CLI commands.\n\nReference: `beans/internal/output/` — `Success()`, `Error()`, `SuccessMultiple()`.\n\n## Requirements\n\n- [ ] `Response` struct: `success: bool`, `message: Option<String>`, `data: Option<T>` (single ishoo), `ishoos: Option<Vec<Ishoo>>` (list), `count: Option<usize>`\n- [ ] Error codes: `not_found`, `validation`, `conflict`, `file_error`\n- [ ] `output_success(ishoo)`, `output_success_multiple(ishoos)`, `output_error(code, message)`\n- [ ] All output goes to stdout as pretty-printed JSON\n- [ ] Every CLI command supports `--json` flag to switch to this mode\n\n## Verification\n\n```bash\ncargo test\n```\n\nUnit tests: verify JSON structure for success and error cases.
