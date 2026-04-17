---
# ish-bpxr
title: Markdown body rendering for show command
status: in-progress
type: task
priority: normal
created_at: 2026-04-17T13:33:58Z
updated_at: 2026-04-17T18:33:35Z
parent: ish-1of2
blocked_by:
    - ish-0iv6
---

## Description\n\nRender ishoo body markdown in the terminal for the `show` command.\n\nReference: `beans/internal/commands/show.go` uses `glamour` for terminal markdown rendering.\n\n## Requirements\n\n- [ ] Use a Rust markdown terminal renderer (e.g. `termimad` or `bat`'s syntax highlighting)\n- [ ] Render headings, bold, italic, code blocks, lists, links in styled terminal output\n- [ ] Word-wrap to 80 columns\n\n## Verification\n\nManual: create an ishoo with rich markdown body, verify `ish show` renders it readably.
