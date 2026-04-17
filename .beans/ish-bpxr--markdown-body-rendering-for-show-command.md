---
# ish-bpxr
title: Markdown body rendering for show command
status: completed
type: task
priority: normal
created_at: 2026-04-17T13:33:58Z
updated_at: 2026-04-17T18:50:06Z
parent: ish-1of2
---

## Description\n\nRender ishoo body markdown in the terminal for the `show` command.\n\nReference: `beans/internal/commands/show.go` uses `glamour` for terminal markdown rendering.\n\n## Requirements\n\n- [x] Use a Rust markdown terminal renderer (e.g. `termimad` or `bat`'s syntax highlighting)\n- [x] Render headings, bold, italic, code blocks, lists, links in styled terminal output\n- [x] Word-wrap to 80 columns\n\n## Verification\n\nManual: create an ishoo with rich markdown body, verify `ish show` renders it readably.\n\nAutomated:\n\n```bash\ncargo test\ncargo fmt --all -- --check\ncargo clippy -- -D warnings\n```\n\n## Summary of Changes\n\n- Added `render_markdown()` and `render_markdown_with_width()` in `src/output/mod.rs` using `termimad` so future `ish show` output can render markdown bodies consistently.\n- Introduced a shared markdown skin for headings, emphasis, lists, inline code, code blocks, and quotes.\n- Added output tests covering common markdown elements, width-constrained wrapping, and blank-body handling.\n- Validation passed: `cargo test`, `cargo fmt --all -- --check`, and `cargo clippy -- -D warnings`.\n\n## Notes for Future Workers\n\n- The renderer currently lives only in `src/output/mod.rs`; the pending `ish show` command can call `render_markdown()` directly for default 80-column wrapping, or `render_markdown_with_width()` if it needs a different width.\n- `termimad` appends trailing newlines as part of formatted text output, so callers may want to trim or place separators accordingly when composing larger command output.
