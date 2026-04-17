---
# ish-gt2k
title: CLI commands
status: completed
type: epic
priority: normal
created_at: 2026-04-17T13:30:45Z
updated_at: 2026-04-17T18:55:33Z
parent: ish-orp4
blocked_by:
    - ish-oby2
    - ish-6ye6
    - ish-s3lu
    - ish-yh74
    - ish-u5co
    - ish-6z0n
    - ish-7rz2
    - ish-3o4l
    - ish-ucno
    - ish-4b45
    - ish-hb0h
---

All CLI commands: init, create, list, show, update, delete, archive, check, roadmap, prime, version. Each command wired to the core engine directly (no GraphQL layer).

## Summary of Changes

- Verified the CLI command epic is complete: init, create, list, show, update, delete, archive, check, roadmap, prime, and version are all implemented and already tracked in completed child beans.
- Confirmed the commands are wired through the CLI without a GraphQL layer, matching the epic scope.
- Ran the full project feedback loop successfully: `cargo test`, `cargo fmt --all -- --check`, and `cargo clippy -- -D warnings`.
