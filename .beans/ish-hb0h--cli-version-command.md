---
# ish-hb0h
title: 'CLI: version command'
status: completed
type: task
priority: normal
created_at: 2026-04-17T13:35:00Z
updated_at: 2026-04-17T15:04:20Z
parent: ish-gt2k
blocked_by:
    - ish-0iv6
---

## Description\n\nImplement `ish version` — print the version number.\n\n## Requirements\n\n- [x] Read version from `Cargo.toml` (via `env!("CARGO_PKG_VERSION")`)\n- [x] Print `ish {version}`\n\n## Verification\n\n```bash\ncargo run -- version\n```

## Summary of Changes

- Added a `version` subcommand to the clap CLI and wired it to print `ish {version}` using `env!("CARGO_PKG_VERSION")`.
- Added a unit test covering the output format and verified `cargo run -- version`, `cargo test`, `cargo fmt --all -- --check`, and `cargo clippy -- -D warnings`.
