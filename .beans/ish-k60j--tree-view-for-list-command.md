---
# ish-k60j
title: Tree view for list command
status: todo
type: task
created_at: 2026-04-17T13:33:58Z
updated_at: 2026-04-17T13:33:58Z
parent: ish-1of2
blocked_by:
    - ish-ffou
    - ish-5cna
---

## Description\n\nImplement tree-view rendering for the `list` command, showing parent-child hierarchy with indent and connectors.\n\nReference: `beans/internal/ui/` — `BuildTree()`, `RenderTree()`.\n\n## Requirements\n\n- [ ] `build_tree(filtered_ishoos, all_ishoos, sort_fn, implicit_statuses)` — build a tree structure from flat list\n  - Filtered ishoos are the "target" nodes\n  - Ancestor ishoos (from all_ishoos) are included as context nodes if they're parents of targets\n  - Context-only ancestors are rendered dimmed\n- [ ] `render_tree(tree, config, max_id_width, has_tags, term_width)` — render to string with:\n  - Tree connectors: `├──`, `└──`, `│`\n  - ID, status badge, type badge, priority badge, title\n  - Tags (if any ishoo has tags)\n  - Truncation based on terminal width\n- [ ] Terminal width detection via `terminal_size` crate, default to 80\n\n## Verification\n\n```bash\ncargo test\n```\n\nUnit tests: tree building with various parent/child topologies, correct connector rendering.
