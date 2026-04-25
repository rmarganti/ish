# Progress

## 2026-04-25
- Completed `ish-loy6` (`TUI: scaffolding, deps, and ish tui CLI entry`).
- Added the initial `ish tui` CLI plumbing:
  - `Commands::Tui` in `src/cli/mod.rs`
  - `src/commands/tui.rs`
  - app dispatch in `src/app/mod.rs`
  - top-level `mod tui;` in `src/main.rs`
- Added the initial TUI module skeleton under `src/tui/` with stub modules:
  `model`, `msg`, `keymap`, `update`, `effect`, `runtime`, `view`, and `theme`.
  `src/tui/mod.rs::run` delegates to `runtime::run`, which is currently a no-op.
- Added `ratatui` and `crossterm` to `Cargo.toml` and refreshed `Cargo.lock`.
- `ish tui` currently requires a normal TTY flow and rejects `--json` with a
  validation error; future TUI work can assume JSON output compatibility is
  intentionally out of scope for this subcommand.
- Verification completed for this foundation step:
  - `mise exec -- cargo test`
  - `mise exec -- cargo run -- tui`
  - `mise exec -- cargo run -- --help | rg "\btui\b"`
  - `mise run ci`
