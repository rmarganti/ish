---
# ish-hb0h
title: 'CLI: version command'
status: todo
type: task
created_at: 2026-04-17T13:35:00Z
updated_at: 2026-04-17T13:35:00Z
parent: ish-gt2k
blocked_by:
    - ish-0iv6
---

## Description\n\nImplement `ish version` — print the version number.\n\n## Requirements\n\n- [ ] Read version from `Cargo.toml` (via `env!("CARGO_PKG_VERSION")`)\n- [ ] Print `ish {version}`\n\n## Verification\n\n```bash\ncargo run -- version\n```
