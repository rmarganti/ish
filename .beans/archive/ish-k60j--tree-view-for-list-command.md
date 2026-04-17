---
# ish-k60j
title: Tree view for list command
status: completed
type: task
priority: normal
created_at: 2026-04-17T13:33:58Z
updated_at: 2026-04-17T16:41:40Z
parent: ish-1of2
blocked_by:
    - ish-ffou
    - ish-5cna
---

## Description\n\nImplement tree-view rendering for the `list` command, showing parent-child hierarchy with indent and connectors.\n\nReference: `beans/internal/ui/` — `BuildTree()`, `RenderTree()`.\n\n## Requirements\n\n- [x] `build_tree(filtered_ishoos, all_ishoos, sort_fn, implicit_statuses)` — build a tree structure from flat list\n  - Filtered ishoos are the "target" nodes\n  - Ancestor ishoos (from all_ishoos) are included as context nodes if they're parents of targets\n  - Context-only ancestors are rendered dimmed\n- [x] `render_tree(tree, config, max_id_width, has_tags, term_width)` — render to string with:\n  - Tree connectors: `├──`, `└──`, `│`\n  - ID, status badge, type badge, priority badge, title\n  - Tags (if any ishoo has tags)\n  - Truncation based on terminal width\n- [x] Terminal width detection via `terminal_size` crate, default to 80\n\n## Verification\n\n```bash\ncargo test\n```\n\nUnit tests: tree building with various parent/child topologies, correct connector rendering.

## Summary of Changes

Added shared tree-building and tree-rendering utilities in `src/output/mod.rs` for upcoming human-readable list output. The tree builder includes filtered issues plus ancestor context, preserves per-level sorting via an injected sorter, and carries implicit status information into rendering.

The renderer now supports tree connectors, styled ID/status/type/priority badges, optional tags, terminal-width truncation, and a terminal width helper backed by `terminal_size` with an 80-column fallback. Added focused unit tests for ancestor-context tree construction, implicit-status rendering, connector output, truncation, and terminal-width defaults.

## Notes for Future Workers

The `list` CLI command is not implemented on this branch yet, so these helpers are intentionally shared utilities waiting to be wired into that command once it lands. The color-dimming behavior itself is exercised indirectly because ANSI state in `colored` is global and can make direct style assertions flaky under the full parallel test suite.

## Verification Notes

- Ran `cargo fmt --all -- --check`
- Ran `cargo test`
- Ran `cargo clippy -- -D warnings`
