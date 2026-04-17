---
# ish-1of2
title: Output & display
status: completed
type: epic
priority: normal
created_at: 2026-04-17T13:30:45Z
updated_at: 2026-04-17T18:56:23Z
parent: ish-orp4
blocked_by:
    - ish-qpqo
    - ish-vb4z
    - ish-k60j
    - ish-bpxr
---

JSON output mode, styled terminal output (colored badges), markdown body rendering, tree view for list, and terminal width detection.

## Summary of Changes

Completed the output and display epic by landing JSON output mode, styled terminal output, markdown body rendering for `show`, and tree rendering for `list`, including terminal-width-aware formatting helpers.

## Notes for Future Workers

- Output formatting now lives in `src/output.rs`, which centralizes JSON envelopes, styled human-readable rendering, tree rendering, markdown rendering, and terminal width detection.
- Command tests in `src/main.rs` and focused output tests cover both JSON and human output paths; extend those tests first when changing display behavior.
- Full validation was rerun after the final integration pass: `cargo test`, `cargo fmt --all -- --check`, and `cargo clippy -- -D warnings` all passed.
