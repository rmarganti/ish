---
# ish-vb4z
title: Styled terminal output
status: completed
type: task
priority: normal
created_at: 2026-04-17T13:33:58Z
updated_at: 2026-04-17T16:26:18Z
parent: ish-1of2
blocked_by:
    - ish-614c
    - ish-0iv6
---

## Description\n\nImplement colored, styled terminal output for human-readable display.\n\nReference: `beans/internal/ui/` — styles for status, type, priority badges, ID rendering, muted text.\n\n## Requirements\n\n- [x] Color mapping: each status, type, and priority has an associated terminal color\n- [x] `render_status(status)` — colored badge, dimmed for archive statuses\n- [x] `render_type(type)` — colored badge\n- [x] `render_priority(priority)` — colored badge\n- [x] `render_id(id)` — bold/highlighted\n- [x] Muted style for secondary text (paths, timestamps, etc.)\n- [x] Bold style for headings\n- [x] Success/danger/warning styles for check/delete output\n\n## Verification\n\nManual: run commands and visually verify styled output. Unit tests for color mapping logic.

## Implementation Notes

- Added terminal styling helpers in `src/output/mod.rs` for status/type/priority badges, bold ID rendering, muted secondary text, headings, and success/danger/warning messages.
- Wired the new success/warning/danger helpers into existing non-JSON CLI paths in `src/main.rs` so current human-readable output already uses the shared style layer.
- Badge-oriented helpers are intentionally available for future `list`, `show`, `check`, and `delete` command rendering; they are unit-tested now even though those commands are not implemented yet.
- ANSI color assertions are covered in unit tests by forcing color output because captured non-TTY command execution suppresses colors during manual CLI runs.

## Summary of Changes

Implemented a reusable terminal styling layer backed by config-defined colors and added tests for palette mapping and rendered labels. Applied the shared styles to existing non-JSON success, warning, and error output so the current CLI already benefits from the new human-readable formatting.

## Verification Notes

- Ran `cargo fmt --all -- --check`
- Ran `cargo test`
- Ran `cargo clippy -- -D warnings`
- Ran `cargo run -- version`
- Ran `cargo run`
