---
# ish-nxj6
title: 'TUI: issue detail view (metadata + scrollable body)'
status: completed
type: task
priority: high
tags:
- tui
created_at: 2026-04-25T03:20:55.658047Z
updated_at: 2026-04-25T04:19:19.283744Z
parent: ish-q6t1
blocked_by:
- ish-8dtp
- ish-5017
---

## Goal
Render the issue detail screen: metadata block + scrollable markdown body.

## Scope
### `src/tui/view/issue_detail.rs`
- `pub fn draw(frame: &mut Frame, area: Rect, model: &Model, state: &DetailState)`.
- Top metadata block: title, id, type (colored), status (colored),
  priority (colored), tags, parent, blocking, blocked_by, updated_at.
- Body: render markdown. Acceptable v1: pass through plain text or use
  `termimad`-rendered text reflowed into ratatui Paragraph; if a clean
  ratatui markdown widget is not available, fall back to monospace text
  with simple heading/code-block styling.
- Scroll respects `state.scroll`.
- Footer keybind hints: `e edit  s status  q back`.

## Files
- `src/tui/view/issue_detail.rs`
- Register the screen in `src/tui/view/mod.rs`.

## Verification
- `mise run ci` passes.
- Manual smoke: opening an issue from the board shows metadata and body;
  `j`/`k` scrolls the body.


## Implementation notes
- Added `src/tui/view/issue_detail.rs` with the first real detail-screen renderer and registered `Screen::IssueDetail(...)` in `src/tui/view.rs` so detail navigation no longer falls back to the generic placeholder screen.
- The detail renderer now shows a metadata block for title, id, colored type/status/priority, tags, parent, blocking, blocked_by, and `updated_at`, then renders the issue body in a scrollable `Paragraph` keyed to `DetailState::scroll`.
- Body rendering stays intentionally lightweight for v1: plain text is preserved, headings get simple bold/underlined styling, fenced-code delimiter lines are dimmed, and empty bodies show a dim `(empty body)` placeholder.
- Added a small missing-issue fallback view so the TUI stays usable if a detail screen remains open after the selected issue disappears from the in-memory cache.
- Added detail-view unit coverage for metadata formatting, body formatting, and a `ratatui::backend::TestBackend` render smoke test.

## Validation
- `mise exec -- cargo test tui::view::issue_detail -- --nocapture`
- `mise run ci`
- `mise exec -- ish check`

## Follow-up notes
- The detail body currently uses a simple `Paragraph` instead of a full markdown widget; if richer markdown support is added later, preserve the existing scroll contract around `DetailState::scroll`.
- The detail screen owns its own footer key hints for now. When `ish-wka9` lands, consolidate shared footer/status-line/help-overlay chrome there instead of duplicating it per screen.
