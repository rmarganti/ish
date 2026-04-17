---
# ish-vb4z
title: Styled terminal output
status: todo
type: task
created_at: 2026-04-17T13:33:58Z
updated_at: 2026-04-17T13:33:58Z
parent: ish-1of2
blocked_by:
    - ish-614c
    - ish-0iv6
---

## Description\n\nImplement colored, styled terminal output for human-readable display.\n\nReference: `beans/internal/ui/` — styles for status, type, priority badges, ID rendering, muted text.\n\n## Requirements\n\n- [ ] Color mapping: each status, type, and priority has an associated terminal color\n- [ ] `render_status(status)` — colored badge, dimmed for archive statuses\n- [ ] `render_type(type)` — colored badge\n- [ ] `render_priority(priority)` — colored badge\n- [ ] `render_id(id)` — bold/highlighted\n- [ ] Muted style for secondary text (paths, timestamps, etc.)\n- [ ] Bold style for headings\n- [ ] Success/danger/warning styles for check/delete output\n\n## Verification\n\nManual: run commands and visually verify styled output. Unit tests for color mapping logic.
